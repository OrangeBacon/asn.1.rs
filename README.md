# asn.1.rs

ASN.1 parser written in rust.

# Standards
- ITU-T X.680 (02/2021) Information technology - Abstract Syntax Notation One (ASN.1): Specification of basic notation

# Todo:
- Unicode identifiers UAX#31
    - XID_Start, XID_End
    - Unicode normalisation form NFC
    - Unicode Security Mechanisms UTS#39
    - Currently identifiers match `[A-Za-z][A-Za-z0-9_$\-]*` with the additional restriction that they cannot contain two consecutive hyphens or end in a hyphen.  Additionally, any hyphens can be hyphens or non-breaking hyphens, which will be treated as identical.