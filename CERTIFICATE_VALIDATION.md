# Certificate Validation for ePassport SOD

## Current Implementation

The current implementation performs basic SOD verification:
- ✅ Calculates SHA-256 hash of DG1 data
- ✅ Verifies hash is present in SOD (integrity check)
- ✅ Validates CMS structure format (ASN.1 markers)

## Production Requirements

For production-grade certificate validation, you need to implement:

### 1. Parse CMS SignedData Structure

The SOD is a CMS (Cryptographic Message Syntax) SignedData structure containing:
- **Version**: Usually 1
- **DigestAlgorithms**: Hash algorithms used (typically SHA-256)
- **EncapsulatedContentInfo**: Contains the signed data (DG hashes)
- **Certificates**: X.509 certificate chain
- **SignerInfos**: Signature information

### 2. Extract Certificate Chain

From the CMS structure, extract:
- **Document Signer (DS) Certificate**: Signs the SOD
- **Country Signing Certificate Authority (CSCA) Certificate**: Root certificate

### 3. Verify Certificate Chain

1. Verify DS certificate is signed by CSCA
2. Verify certificate chain is complete
3. Check certificate validity dates
4. Verify certificate hasn't been revoked (CRL/OCSP)

### 4. Verify Signature

1. Extract the signature from SignerInfos
2. Extract the public key from DS certificate
3. Verify the signature using the public key
4. Verify signed attributes match the actual data

### 5. Verify DG Hash

1. Extract DG1 hash from signed attributes (messageDigest)
2. Compare with calculated SHA-256 hash of DG1 data
3. Verify all required data groups are present

## Implementation Options

### Option 1: Full Verification in zkVM

Use libraries like:
- `der` or `asn1_rs` for ASN.1 parsing
- `x509-parser` for certificate parsing
- `ring` or `rustls` for signature verification

**Pros**: Complete verification in zero-knowledge
**Cons**: Complex, computationally expensive, may hit zkVM limits

### Option 2: Hybrid Approach

1. Pre-verify certificates on host side
2. Pass verification results to zkVM
3. Verify hash integrity in zkVM

**Pros**: Simpler, faster, more practical
**Cons**: Requires trust in host verification

### Option 3: Certificate Validation Service

1. Verify certificates in a trusted service
2. Generate a certificate validation proof
3. Verify the proof in zkVM

**Pros**: Best of both worlds
**Cons**: Requires additional infrastructure

## Recommended Next Steps

1. **Short term**: Enhance current hash verification (done ✅)
2. **Medium term**: Add CMS structure parsing to extract certificates
3. **Long term**: Implement full certificate chain verification

## Resources

- [ICAO 9303 Standard](https://www.icao.int/publications/pages/publication.aspx?docnum=9303) - ePassport specifications
- [RFC 5652](https://tools.ietf.org/html/rfc5652) - CMS specification
- [RFC 5280](https://tools.ietf.org/html/rfc5280) - X.509 certificate specification

## Example CMS Parsing (Pseudocode)

```rust
// Parse CMS SignedData
let cms = parse_cms_signed_data(sod_data)?;

// Extract certificates
let certificates = cms.certificates?;
let ds_cert = find_ds_certificate(certificates)?;
let csca_cert = find_csca_certificate(certificates)?;

// Verify certificate chain
verify_certificate_chain(ds_cert, csca_cert, trusted_csca_roots)?;

// Extract signed attributes
let signed_attrs = cms.signer_infos[0].signed_attributes?;
let dg1_hash_from_sod = extract_dg1_hash(signed_attrs)?;

// Verify hash matches
let dg1_hash_calculated = sha256(dg1_data);
assert_eq!(dg1_hash_from_sod, dg1_hash_calculated);

// Verify signature
let signature = cms.signer_infos[0].signature;
let public_key = ds_cert.public_key();
verify_signature(signed_attrs, signature, public_key)?;
```

