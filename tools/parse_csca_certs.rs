//! Utility to parse ICAO PKD LDIF file and extract CSCA certificates
//! This creates a JSON file with trusted CSCA certificates that can be used
//! for certificate chain verification in the zkVM circuit

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::io::{BufRead, BufReader};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrustedCSCACert {
    pub country_code: String,
    pub certificate_hash: [u8; 32],
    pub certificate_der: Vec<u8>,
    pub serial_number: String,
    pub common_name: String,
    // Additional ICAO PKD attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkd_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkd_conformance_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkd_conformance_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkd_conformance_policy: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub object_classes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinguished_name: Option<String>,
    // Store any other attributes that might be present
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub other_attributes: std::collections::HashMap<String, String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.ldif> <output.json>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    println!("Parsing LDIF file: {}", input_file);
    let certs = parse_ldif_file(input_file);
    println!("Found {} CSCA certificates", certs.len());

    // Group by country
    let mut by_country: std::collections::HashMap<String, Vec<TrustedCSCACert>> = std::collections::HashMap::new();
    for cert in &certs {
        by_country.entry(cert.country_code.clone())
            .or_insert_with(Vec::new)
            .push(cert.clone());
    }

    println!("Certificates by country:");
    for (country, country_certs) in &by_country {
        println!("  {}: {} certificates", country, country_certs.len());
    }

    // Save to JSON
    let json = serde_json::to_string_pretty(&certs).unwrap();
    fs::write(output_file, json).expect("Failed to write output file");
    println!("Saved {} certificates to {}", certs.len(), output_file);
}

fn parse_ldif_file(filename: &str) -> Vec<TrustedCSCACert> {
    let file = fs::File::open(filename).expect("Failed to open LDIF file");
    let reader = BufReader::new(file);

    let mut certificates = Vec::new();
    let mut current_entry: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut current_country: Option<String> = None;
    let mut current_cert_base64 = String::new();
    let mut in_certificate = false;
    let mut current_dn: Option<String> = None;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let trimmed = line.trim();
        
        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }
        
        // Check if line starts with space (continuation) - LDIF format uses space for continuation
        if line.starts_with(' ') || line.starts_with('\t') {
            if in_certificate {
                // Continuation of base64 certificate data
                // Remove leading space/tab, but preserve the rest (including any trailing spaces that might be part of base64)
                // Actually, base64 shouldn't have spaces, so we can trim
                let continuation = line.trim_start();
                if !continuation.is_empty() {
                    current_cert_base64.push_str(continuation);
                }
            } else if let Some(ref dn) = current_dn {
                // Continuation of DN
                current_dn = Some(format!("{} {}", dn, line.trim_start()));
            }
            continue;
        }
        
        // Empty line marks end of entry
        if trimmed.is_empty() {
            if !current_cert_base64.is_empty() {
                // Extract country from DN if not already set
                if current_country.is_none() {
                    if let Some(ref dn) = current_dn {
                        current_country = extract_country_from_dn(dn);
                    }
                }
                
                if let Some(cert) = parse_entry(&current_entry, &current_country, &current_cert_base64, &current_dn) {
                    certificates.push(cert);
                }
            }
            current_entry.clear();
            current_cert_base64.clear();
            current_country = None;
            current_dn = None;
            in_certificate = false;
            continue;
        }

        // Parse LDIF attribute (format: "key: value" or "key:: base64value")
        if let Some(colon_pos) = trimmed.find(':') {
            let key = &trimmed[..colon_pos];
            let value_part = &trimmed[colon_pos + 1..];
            
            // If we were collecting a certificate and encounter a new attribute, stop collecting
            if in_certificate && key != "userCertificate;binary" {
                in_certificate = false;
            }
            
            // Check for double colon (base64 encoding indicator)
            // LDIF format: "key:: base64value" where space after :: is optional
            let (is_base64, value) = if value_part.starts_with(':') {
                // Double colon means base64 - skip the second colon and any leading whitespace
                let base64_value = value_part[1..].trim_start();
                (true, base64_value)
            } else {
                (false, value_part.trim())
            };

            match key {
                "dn" => {
                    current_dn = Some(value.to_string());
                    // Try to extract country from DN
                    current_country = extract_country_from_dn(value);
                }
                "c" => {
                    current_country = Some(value.to_string());
                }
                "userCertificate;binary" => {
                    in_certificate = true;
                    current_cert_base64 = value.to_string();
                }
                "sn" => {
                    current_entry.entry("serialNumber".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "cn" => {
                    current_entry.entry("commonName".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "objectClass" => {
                    current_entry.entry("objectClass".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "pkdVersion" => {
                    current_entry.entry("pkdVersion".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "pkdConformanceCode" => {
                    current_entry.entry("pkdConformanceCode".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "pkdConformanceText" => {
                    current_entry.entry("pkdConformanceText".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "pkdConformancePolicy" => {
                    current_entry.entry("pkdConformancePolicy".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                "o" => {
                    current_entry.entry("organization".to_string())
                        .or_insert_with(Vec::new)
                        .push(value.to_string());
                }
                _ => {
                    if !is_base64 {
                        // Store other attributes (take first value if multiple)
                        current_entry.entry(key.to_string())
                            .or_insert_with(Vec::new)
                            .push(value.to_string());
                    }
                }
            }
        }
    }

    // Handle last entry
    if !current_cert_base64.is_empty() {
        if current_country.is_none() {
            if let Some(ref dn) = current_dn {
                current_country = extract_country_from_dn(dn);
            }
        }
        if let Some(cert) = parse_entry(&current_entry, &current_country, &current_cert_base64, &current_dn) {
            certificates.push(cert);
        }
    }

    certificates
}

fn extract_country_from_dn(dn: &str) -> Option<String> {
    // DN format: "cn=...,c=NZ,dc=..."
    // Extract country code from DN
    for part in dn.split(',') {
        let part = part.trim();
        if part.starts_with("c=") {
            return Some(part[2..].to_string());
        }
    }
    None
}

fn parse_entry(
    entry: &std::collections::HashMap<String, Vec<String>>,
    country: &Option<String>,
    cert_base64: &str,
    dn: &Option<String>,
) -> Option<TrustedCSCACert> {
    if cert_base64.is_empty() {
        return None;
    }

    // Clean base64 string (remove all whitespace including newlines, spaces, tabs)
    let cleaned_base64: String = cert_base64
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    
    if cleaned_base64.len() < 100 {
        // Too short to be a valid certificate
        eprintln!("Warning: Base64 string too short ({} chars)", cleaned_base64.len());
        return None;
    }

    // Decode base64 certificate
    // The base64 crate handles padding automatically, but we need to ensure valid base64 characters
    // Base64 alphabet: A-Z, a-z, 0-9, +, /, and = for padding
    let valid_chars: String = cleaned_base64
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
        .collect();
    
    if valid_chars.len() < cleaned_base64.len() {
        eprintln!("Warning: Found invalid base64 characters, cleaned {} -> {} chars", 
            cleaned_base64.len(), valid_chars.len());
    }
    
    // Decode with proper padding handling
    let cert_der = match general_purpose::STANDARD.decode(&valid_chars) {
        Ok(cert) => cert,
        Err(e) => {
            // Try to fix padding issues
            let mut fixed = valid_chars.clone();
            // Remove all padding first
            while fixed.ends_with('=') {
                fixed.pop();
            }
            // Add proper padding
            let remainder = fixed.len() % 4;
            if remainder != 0 {
                let needed_padding = 4 - remainder;
                fixed.push_str(&"=".repeat(needed_padding.min(2)));
            }
            
            match general_purpose::STANDARD.decode(&fixed) {
                Ok(cert) => cert,
                Err(e2) => {
                    eprintln!("Warning: Failed to decode certificate: {} (tried fixing padding: {})", 
                        e, e2);
                    eprintln!("  Base64 length: {}, Valid chars: {}", cleaned_base64.len(), valid_chars.len());
                    eprintln!("  First 100 chars: {}", &cleaned_base64.chars().take(100).collect::<String>());
                    return None;
                }
            }
        }
    };
    
    if cert_der.len() < 100 {
        // Decoded certificate too short
        return None;
    }

    // Calculate certificate hash
    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let cert_hash = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(cert_hash.as_slice());

    let country_code = country.clone().unwrap_or_else(|| "UNKNOWN".to_string());
    
    // Extract single-value attributes (take first if multiple)
    let serial_number = entry.get("serialNumber")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| "".to_string());
    
    let common_name = entry.get("commonName")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| "".to_string());
    
    // Extract optional PKD attributes
    let pkd_version = entry.get("pkdVersion")
        .and_then(|v| v.first())
        .cloned();
    
    let pkd_conformance_code = entry.get("pkdConformanceCode")
        .and_then(|v| v.first())
        .cloned();
    
    let pkd_conformance_text = entry.get("pkdConformanceText")
        .and_then(|v| v.first())
        .cloned();
    
    let pkd_conformance_policy = entry.get("pkdConformancePolicy")
        .and_then(|v| v.first())
        .cloned();
    
    // Extract object classes (can have multiple values)
    let object_classes = entry.get("objectClass")
        .cloned()
        .unwrap_or_else(Vec::new);
    
    let organization = entry.get("organization")
        .and_then(|v| v.first())
        .cloned();
    
    // Store other attributes (excluding ones we've already extracted)
    let excluded_keys: std::collections::HashSet<&str> = [
        "serialNumber", "commonName", "objectClass", "pkdVersion",
        "pkdConformanceCode", "pkdConformanceText", "pkdConformancePolicy",
        "organization"
    ].iter().cloned().collect();
    
    let mut other_attributes = std::collections::HashMap::new();
    for (key, values) in entry {
        if !excluded_keys.contains(key.as_str()) && !values.is_empty() {
            // Take first value for other attributes
            if let Some(first_value) = values.first() {
                other_attributes.insert(key.clone(), first_value.clone());
            }
        }
    }

    Some(TrustedCSCACert {
        country_code,
        certificate_hash: hash_array,
        certificate_der: cert_der,
        serial_number,
        common_name,
        pkd_version,
        pkd_conformance_code,
        pkd_conformance_text,
        pkd_conformance_policy,
        object_classes,
        organization,
        distinguished_name: dn.clone(),
        other_attributes,
    })
}

