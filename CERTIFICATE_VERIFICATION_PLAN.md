# Certificate Verification Implementation Plan

## Current Status
The circuit currently uses a heuristic check that only verifies if "DIGERCIC CSCA" appears anywhere in certificate data. This is NOT cryptographically secure.

## Requirements for Proper Certificate Verification

### 1. Parse X.509 Certificate Structure
X.509 certificates are ASN.1 DER-encoded structures with this format:

```
Certificate ::= SEQUENCE {
    tbsCertificate       TBSCertificate,
    signatureAlgorithm   AlgorithmIdentifier,
    signatureValue       BIT STRING
}

TBSCertificate ::= SEQUENCE {
    version              [0] EXPLICIT Version DEFAULT v1,
    serialNumber         CertificateSerialNumber,
    signature            AlgorithmIdentifier,
    issuer               Name,
    validity             Validity,
    subject              Name,
    subjectPublicKeyInfo SubjectPublicKeyInfo,
    ...
}

Name ::= SEQUENCE OF RelativeDistinguishedName
RelativeDistinguishedName ::= SET OF AttributeTypeAndValue
AttributeTypeAndValue ::= SEQUENCE {
    type  OBJECT IDENTIFIER,
    value ANY
}
```

### 2. Required Verification Steps

#### Step 1: Extract DS Certificate from SOD
- Parse CMS SignedData structure
- Extract certificates field (tag 0xA0)
- Identify the DS (Document Signer) certificate

#### Step 2: Extract CSCA Certificate Information
- Parse the trusted CSCA certificate DER
- Extract subject Distinguished Name (DN)
- Extract public key and algorithm

#### Step 3: Verify Issuer-Subject Match
- Parse DS certificate's issuer DN
- Parse CSCA certificate's subject DN
- Compare the DNs (must match exactly)

#### Step 4: Verify Cryptographic Signature
- Extract DS certificate's TBSCertificate (To Be Signed portion)
- Extract DS certificate's signature value
- Extract CSCA certificate's public key
- Verify signature: `verify(signature, TBSCertificate, CSCA_public_key)`

#### Step 5: Verify Certificate Validity
- Check DS certificate not expired
- Check CSCA certificate not expired
- (Optional) Check revocation status

## Implementation Approach

### Option 1: Full Implementation in zkVM (Recommended for Security)
Use existing Rust crates that work in no_std environments:
- `der` crate for ASN.1 parsing ✅ (already in Cargo.toml)
- `x509-parser` crate ✅ (already in Cargo.toml)
- `rsa` crate for RSA signature verification
- `p256` crate for ECDSA signature verification (if needed)

**Pros:**
- Full cryptographic verification in zero-knowledge
- Maximum security
- Trustless verification

**Cons:**
- Computationally expensive in zkVM
- May significantly increase proof generation time
- Requires implementing signature verification algorithms

### Option 2: Hybrid Approach
1. Pre-verify certificates on host side
2. Pass verification witness to zkVM
3. Verify witness in zkVM

**Pros:**
- Faster proof generation
- Simpler zkVM code

**Cons:**
- Requires trust in host verification
- More complex architecture

### Option 3: Simplified but Secure Approach (Recommended for MVP)
1. Properly parse certificate structure to extract issuer DN
2. Compare issuer DN with trusted CSCA subject DN (exact match)
3. Skip full signature verification initially
4. Add signature verification in next iteration

**Pros:**
- More secure than current heuristic
- Reasonable performance
- Incremental improvement path

**Cons:**
- Still not full cryptographic verification
- Trust assumption: if issuer DN matches, certificate is valid

## Recommended Implementation Plan

### Phase 1: DN Extraction and Comparison (Implement Now)
1. Parse DS certificate ASN.1 structure
2. Extract issuer DN from DS certificate
3. Extract subject DN from trusted CSCA certificate
4. Compare DNs for exact match

### Phase 2: Signature Verification (Next Iteration)
1. Add RSA signature verification
2. Extract public key from CSCA certificate
3. Extract signature from DS certificate
4. Verify signature cryptographically

### Phase 3: Full PKI Validation (Production)
1. Add certificate expiry checks
2. Add revocation checking
3. Support multiple signature algorithms (RSA, ECDSA)
4. Handle certificate extensions

## Code Structure

```rust
// Phase 1: DN Extraction
fn extract_issuer_dn(cert_der: &[u8]) -> Option<Vec<u8>> { ... }
fn extract_subject_dn(cert_der: &[u8]) -> Option<Vec<u8>> { ... }
fn compare_dns(dn1: &[u8], dn2: &[u8]) -> bool { ... }

// Phase 2: Signature Verification
fn extract_tbs_certificate(cert_der: &[u8]) -> Option<&[u8]> { ... }
fn extract_signature(cert_der: &[u8]) -> Option<&[u8]> { ... }
fn extract_public_key(cert_der: &[u8]) -> Option<PublicKey> { ... }
fn verify_signature(tbs: &[u8], sig: &[u8], pubkey: &PublicKey) -> bool { ... }
```

## Dependencies Required

### For DN Parsing (Phase 1)
- `der` ✅ (already added)
- `x509-parser` ✅ (already added)

### For Signature Verification (Phase 2)
- `rsa` (for RSA signature verification)
- `sha2` ✅ (already added)
- `signature` (trait for signature verification)

### zkVM Compatibility Check
Need to verify these crates work in SP1's zkVM environment:
- Check if they support `no_std`
- Check if they have dependencies on std-only features
- May need to use alternative implementations

## Testing Strategy

1. **Unit Tests**: Test each parsing function separately
2. **Integration Tests**: Test full verification flow
3. **Test Vectors**: Use known good/bad certificates
4. **Performance Tests**: Measure zkVM cycle count

## Security Considerations

1. **DN Comparison**: Must be exact match (case-sensitive, order-sensitive)
2. **Signature Algorithms**: Support common algorithms (RSA-SHA256, ECDSA-SHA256)
3. **Certificate Chain**: Verify entire chain up to root
4. **Validity Period**: Check current date is within validity
5. **Revocation**: Check CRL/OCSP (optional for MVP)

## Next Steps

1. Implement DN extraction functions
2. Test with Ecuador CSCA and DS certificates
3. Add comprehensive error handling
4. Document security assumptions
5. Plan Phase 2 implementation

