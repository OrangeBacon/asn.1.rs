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

# Limitations:
The following deviations from standards are made within this parser:
- X.680 OID-IRI values are specified with double quote tokens as their own production that is separate from character strings, however it is ambiguous whether a given double quote should be parsed as a character string or an OID-IRI until typechecking.

    The example included below is ambiguous in the standard, as it could be parsed as either the assignment of character strings to both of `Foo` and `Bar` or alternatively it could be the assignment of one OID-IRI value containing a multi-line comment to `Foo`.
    ```asn1
    Foo MyType ::= "/a /*comment"
    Bar MyType ::= "*//b"
    ```
    This ambiguity cannot be resolved until MyType is resolved to either a character string type or an OID-IRI.
    Therefore, this parser has taken the decision to always parse the above example as two assignments of character string literals and never as an OID-IRI value.