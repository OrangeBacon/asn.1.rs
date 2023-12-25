# asn.1.rs

ASN.1 parser written in rust.

# Standards
- ITU-T X.680 (02/2021) Information technology - Abstract Syntax Notation One (ASN.1): Specification of basic notation

## Secondary References
- ITU-T X.660 (07/2011) Information technology – Procedures for the operation of object identifier registration authorities: General procedures and top arcs of the international object identifier tree
- RFC 3987 Internationalized Resource Identifiers (IRIs)

# Todo:
- Unicode identifiers UAX#31
    - XID_Start, XID_End
    - Unicode normalisation form NFC
    - Unicode Security Mechanisms UTS#39
    - Currently identifiers match `[A-Za-z][A-Za-z0-9_$\-]*` with the additional restriction that they cannot contain two consecutive hyphens or end in a hyphen.  Additionally, any hyphens can be hyphens or non-breaking hyphens, which will be treated as identical.