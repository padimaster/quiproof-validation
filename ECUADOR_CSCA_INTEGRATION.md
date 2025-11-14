# Ecuador CSCA Certificate Integration

## Summary

Ecuador's CSCA (Country Signing Certificate Authority) certificate has been successfully extracted from the ICAO PKD Master List and integrated into the circuit verification system.

## Certificate Details

- **Country Code**: EC
- **Serial Number**: 45E81526E4536A249A3621BC464F2875
- **Subject/Issuer**: `C=EC, O=DIRECCION GENERAL DE REGISTRO CIVIL IDENTIFICACION Y CEDULACION, OU=DIGERCIC CSCA, serialNumber=1, CN=CSCA`
- **Certificate Hash (SHA-256)**: `f62077103edb9ed4055bb812075537885c0cfd439e53119ebaeccc34c69051df`
- **Validity**: Aug 24, 2020 to Nov 24, 2035
- **Key Size**: 4096-bit RSA
- **Certificate Size**: 2046 bytes (DER format)

## Source

The certificate was extracted from:
- **File**: `certificates/icaopkd-002-complete-000329.ldif` (ICAO PKD Master List)
- **Entry**: Ecuador's Master List entry (PKD Version 161)
- **Method**: Extracted from `pkdMasterListContent` (CMS SignedData structure)

## Files Created

1. **`trusted_csca_certs_ecuador.json`** - Ecuador CSCA certificate in circuit format
2. **`/tmp/ecuador_csca_cert.der`** - Raw DER certificate (for reference)
3. **`/tmp/trusted_csca_certs_with_ecuador.json`** - Combined list with all 29,751 certificates (including Ecuador)

## Usage in Circuit

To use Ecuador's CSCA certificate in the circuit verification:

### Option 1: Use Ecuador-only certificate file

Update your `payload.json`:

```json
{
  "dg1_base64": "...",
  "sod_base64": "...",
  "age_to_verify": 18,
  "current_date": [2024, 1, 1],
  "trusted_csca_certs_file": "trusted_csca_certs_ecuador.json"
}
```

### Option 2: Use combined certificate list

If you want to verify against all countries including Ecuador:

```json
{
  "dg1_base64": "...",
  "sod_base64": "...",
  "age_to_verify": 18,
  "current_date": [2024, 1, 1],
  "trusted_csca_certs_file": "trusted_csca_certs_with_ecuador.json"
}
```

### Option 3: Add to existing trusted certificates

You can merge Ecuador's certificate with your existing trusted certificates list by adding the entry from `trusted_csca_certs_ecuador.json` to your existing JSON file.

## Circuit Verification

The circuit's `verify_certificate_chain` function will:

1. Extract certificates from the SOD (Signed Object Document)
2. Calculate SHA-256 hashes of extracted certificates
3. Match against trusted CSCA certificates (including Ecuador's)
4. Verify the certificate chain for Ecuadorian passports

## Testing

To test with an Ecuadorian passport:

```bash
cd circuits/script
cargo run --release -- --execute --payload ../payload.json
```

Make sure your `payload.json` references the Ecuador certificate file and contains valid Ecuadorian passport data (DG1 and SOD).

## Notes

- The certificate is self-signed (issuer = subject)
- The certificate is valid until November 24, 2035
- The certificate uses RSA-PSS signature algorithm with SHA-256
- The certificate includes CRL (Certificate Revocation List) distribution points

## Verification

You can verify the certificate independently:

```bash
openssl x509 -in /tmp/ecuador_csca_cert.der -inform DER -text -noout
```

Or check the hash:

```bash
sha256sum /tmp/ecuador_csca_cert.der
# Should output: f62077103edb9ed4055bb812075537885c0cfd439e53119ebaeccc34c69051df
```
