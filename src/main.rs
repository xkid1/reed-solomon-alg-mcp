mod rs;

use rs::Gf;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

const SERVER_NAME: &str = "reed-solomon-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let clean: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',')
        .collect::<String>()
        .trim_start_matches("0x")
        .to_string();
    if clean.len() & 1 == 1 {
        return Err("hex string must have an even number of digits".into());
    }
    (0..clean.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&clean[i..i + 2], 16)
                .map_err(|_| format!("invalid hex near '{}'", &clean[i..i + 2]))
        })
        .collect()
}

fn get_data(args: &Value, key: &str) -> Result<Vec<u8>, String> {
    let s = args
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or(format!("missing string argument '{}'", key))?;
    match args
        .get("encoding")
        .and_then(|v| v.as_str())
        .unwrap_or("utf8")
    {
        "utf8" => Ok(s.as_bytes().to_vec()),
        "hex" => parse_hex(s),
        other => Err(format!("unknown encoding '{}', use 'utf8' or 'hex'", other)),
    }
}

fn get_usize(args: &Value, key: &str) -> Result<usize, String> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .ok_or(format!("missing non-negative integer argument '{}'", key))
}

fn get_u8(args: &Value, key: &str) -> Result<u8, String> {
    let v = get_usize(args, key)?;
    if v > 255 {
        Err(format!("'{}' must be in 0..=255", key))
    } else {
        Ok(v as u8)
    }
}

fn call_tool(gf: &Gf, name: &str, a: &Value) -> Result<Value, String> {
    match name {
        "rs_encode" => {
            let msg = get_data(a, "data")?;
            let nsym = get_usize(a, "parity_symbols")?;
            let cw = gf.encode(&msg, nsym)?;
            Ok(json!({
                "codeword_hex": hex(&cw),
                "parity_hex": hex(&cw[msg.len()..]),
                "n": cw.len(), "k": msg.len(), "parity_symbols": nsym,
                "max_correctable_errors": nsym / 2,
                "max_correctable_erasures": nsym
            }))
        }
        "rs_decode" => {
            let cw = get_data(a, "codeword")?;
            let nsym = get_usize(a, "parity_symbols")?;
            let d = gf.decode(&cw, nsym)?;
            Ok(json!({
                "message_hex": hex(&d.message),
                "message_utf8": String::from_utf8(d.message.clone()).ok(),
                "corrected_codeword_hex": hex(&d.codeword),
                "errors_corrected": d.error_positions.len(),
                "error_positions": d.error_positions,
                "error_magnitudes_hex": d.error_magnitudes.iter().map(|m| format!("{:02x}", m)).collect::<Vec<_>>()
            }))
        }
        "rs_syndromes" => {
            let cw = get_data(a, "codeword")?;
            let nsym = get_usize(a, "parity_symbols")?;
            let s = gf.syndromes(&cw, nsym);
            let s = &s[1..];
            Ok(json!({
                "syndromes_hex": s.iter().map(|x| format!("{:02x}", x)).collect::<Vec<_>>(),
                "valid_codeword": s.iter().all(|&x| x == 0)
            }))
        }
        "rs_generator_polynomial" => {
            let nsym = get_usize(a, "parity_symbols")?;
            if nsym == 0 || nsym > 254 {
                return Err("parity_symbols must be in 1..=254".into());
            }
            let g = gf.generator_poly(nsym);
            Ok(json!({
                "coefficients_highest_degree_first": g,
                "degree": nsym,
                "roots": (0..nsym).map(|i| format!("alpha^{}", i)).collect::<Vec<_>>()
            }))
        }
        "rs_code_parameters" => {
            let (n, k) = match (
                a.get("n").and_then(|v| v.as_u64()),
                a.get("k").and_then(|v| v.as_u64()),
            ) {
                (Some(n), Some(k)) => (n as usize, k as usize),
                _ => {
                    let k = get_usize(a, "message_length")?;
                    let p = get_usize(a, "parity_symbols")?;
                    (k + p, k)
                }
            };
            if n > 255 || k == 0 || k >= n {
                return Err("require 0 < k < n <= 255 for RS over GF(256)".into());
            }
            let p = n - k;
            Ok(json!({
                "n": n, "k": k, "parity_symbols": p,
                "notation": format!("RS({},{})", n, k),
                "symbol_bits": 8,
                "min_distance": p + 1,
                "max_correctable_errors": p / 2,
                "max_correctable_erasures": p,
                "errors_and_erasures_rule": "2*errors + erasures <= parity_symbols",
                "code_rate": k as f64 / n as f64,
                "overhead_percent": 100.0 * p as f64 / k as f64,
                "max_burst_bits_correctable_approx": ((p / 2).saturating_sub(1)) * 8 + 1,
                "shortened": n < 255
            }))
        }
        "rs_inject_errors" => {
            let mut cw = get_data(a, "codeword")?;
            let mask = if a.get("xor_mask").is_some() {
                get_u8(a, "xor_mask")?
            } else {
                0xFF
            };
            if mask == 0 {
                return Err("xor_mask must be non-zero".into());
            }
            let positions = a
                .get("positions")
                .and_then(|v| v.as_array())
                .ok_or("missing array 'positions'")?;
            for p in positions {
                let i = p.as_u64().ok_or("positions must be integers")? as usize;
                if i >= cw.len() {
                    return Err(format!("position {} out of range (len {})", i, cw.len()));
                }
                cw[i] ^= mask;
            }
            Ok(json!({ "corrupted_codeword_hex": hex(&cw), "positions": positions }))
        }
        "gf256_math" => {
            let op = a.get("op").and_then(|v| v.as_str()).ok_or("missing 'op'")?;
            let x = get_u8(a, "a")?;
            let y = || get_u8(a, "b");
            let r: Value = match op {
                "add" => json!(x ^ y()?),
                "mul" => json!(gf.mul(x, y()?)),
                "div" => json!(gf.div(x, y()?)?),
                "pow" => json!(gf.pow(x, y()? as u32)),
                "inverse" => json!(gf.inv(x)?),
                "log" => json!(gf.log_of(x).ok_or("log of 0 is undefined")?),
                "exp" => json!(gf.alpha(x as usize)),
                _ => return Err("op must be one of add, mul, div, pow, inverse, log, exp".into()),
            };
            Ok(
                json!({ "result": r, "field": "GF(2^8)", "primitive_polynomial": "0x11d (x^8+x^4+x^3+x^2+1)", "alpha": 2 }),
            )
        }
        _ => Err(format!("unknown tool '{}'", name)),
    }
}

fn tools() -> Value {
    let enc = json!({"type":"string","enum":["utf8","hex"],"description":"How to interpret the data string. Default utf8."});
    let nsym = json!({"type":"integer","minimum":1,"maximum":254,"description":"Number of parity (check) symbols (bytes). Corrects up to floor(n/2) byte errors."});
    json!([
        {"name":"rs_encode","description":"Reed-Solomon encode data (GF(256), systematic). Returns message followed by parity bytes. Total length must be <= 255.",
         "inputSchema":{"type":"object","properties":{"data":{"type":"string"},"encoding":enc,"parity_symbols":nsym},"required":["data","parity_symbols"]}},
        {"name":"rs_decode","description":"Decode a Reed-Solomon codeword, locating and correcting up to parity_symbols/2 corrupted bytes (Berlekamp-Massey, Chien search, Forney). Reports positions and magnitudes.",
         "inputSchema":{"type":"object","properties":{"codeword":{"type":"string"},"encoding":enc,"parity_symbols":nsym},"required":["codeword","parity_symbols"]}},
        {"name":"rs_syndromes","description":"Compute syndromes of a received word. All-zero syndromes mean a valid codeword.",
         "inputSchema":{"type":"object","properties":{"codeword":{"type":"string"},"encoding":enc,"parity_symbols":nsym},"required":["codeword","parity_symbols"]}},
        {"name":"rs_generator_polynomial","description":"Return the RS generator polynomial g(x)=prod(x-alpha^i), i=0..nsym-1 over GF(256).",
         "inputSchema":{"type":"object","properties":{"parity_symbols":nsym},"required":["parity_symbols"]}},
        {"name":"rs_code_parameters","description":"Explain RS(n,k) code capabilities: error/erasure capacity, rate, overhead, minimum distance. Give either n and k, or message_length and parity_symbols.",
         "inputSchema":{"type":"object","properties":{"n":{"type":"integer"},"k":{"type":"integer"},"message_length":{"type":"integer"},"parity_symbols":{"type":"integer"}}}},
        {"name":"rs_inject_errors","description":"Corrupt a codeword for testing by XOR-ing bytes at given 0-based positions with a mask (default 0xFF).",
         "inputSchema":{"type":"object","properties":{"codeword":{"type":"string"},"encoding":enc,"positions":{"type":"array","items":{"type":"integer"}},"xor_mask":{"type":"integer","minimum":1,"maximum":255}},"required":["codeword","positions"]}},
        {"name":"gf256_math","description":"Galois field GF(2^8) arithmetic (poly 0x11d, alpha=2): add, mul, div, pow, inverse, log, exp.",
         "inputSchema":{"type":"object","properties":{"op":{"type":"string","enum":["add","mul","div","pow","inverse","log","exp"]},"a":{"type":"integer","minimum":0,"maximum":255},"b":{"type":"integer","minimum":0,"maximum":255}},"required":["op","a"]}}
    ])
}

fn handle(gf: &Gf, req: &Value) -> Option<Value> {
    let method = req.get("method")?.as_str()?;
    let id = req.get("id").cloned();
    // Notifications have no id and get no response.
    let id = id?;
    let ok = |result: Value| json!({"jsonrpc":"2.0","id":id,"result":result});
    let err = |code: i64, msg: String| json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":msg}});
    Some(match method {
        "initialize" => {
            let v = req["params"]["protocolVersion"]
                .as_str()
                .unwrap_or("2025-06-18");
            ok(json!({
                "protocolVersion": v,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
                "instructions": "Reed-Solomon assistant over GF(256). Use rs_encode/rs_decode for data, rs_code_parameters to size a code, rs_inject_errors to test."
            }))
        }
        "ping" => ok(json!({})),
        "tools/list" => ok(json!({"tools": tools()})),
        "tools/call" => {
            let name = req["params"]["name"].as_str().unwrap_or("");
            let args = req["params"].get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(gf, name, &args) {
                Ok(v) => ok(json!({
                    "content":[{"type":"text","text":serde_json::to_string_pretty(&v).unwrap()}],
                    "structuredContent": v, "isError": false })),
                Err(e) if e.starts_with("unknown tool") => err(-32602, e),
                Err(e) => ok(json!({"content":[{"type":"text","text":e}],"isError":true})),
            }
        }
        _ => err(-32601, format!("method not found: {}", method)),
    })
}

fn main() {
    let gf = Gf::new();
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let resp = match serde_json::from_str::<Value>(&line) {
            Ok(req) => handle(&gf, &req),
            Err(e) => Some(
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":format!("parse error: {}", e)}}),
            ),
        };
        if let Some(r) = resp {
            let _ = writeln!(out, "{}", r);
            let _ = out.flush();
        }
    }
}
