# asn.1.rs

ASN.1 parser written in rust.

# Usage


# Standards
- ITU-T X.680 (02/2021) Information technology - Abstract Syntax Notation One (ASN.1): Specification of basic notation
- ITU-T X.681 (02/2021) Information technology - Abstract Syntax Notation One (ASN.1): Information object
specification
- ITU-T X.682 (02/2021) Information technology - Abstract Syntax Notation One (ASN.1): Constraint specification
- ITU-T X.683 (02/2021) Information technology - Abstract Syntax Notation One (ASN.1): Parameterization of ASN.1 specifications

## Secondary References
- ITU-T X.660 (07/2011) Information technology – Procedures for the operation of object identifier registration authorities: General procedures and top arcs of the international object identifier tree
- RFC 3987 Internationalized Resource Identifiers (IRIs)
- RFC 5891 Internationalized Domain Names in Applications (IDNA): Protocol
- The Unicode Standard, Version 15.1.0
    - The Unicode Standard is used in all places within other standards instead of ISO/IEC 10646.  This should not affect the behaviour of any input as the two standards are compatible.

# Limitations:
The following deviations from standards are made within this parser:
- X.680 OID-IRI values are specified with double quote tokens as their own production that is separate from character strings, however it is ambiguous whether a given double quote should be parsed as a character string or an OID-IRI until typechecking.

    The example included below is ambiguous in the standard, as it could be parsed as either the assignment of character strings to both of `Foo` and `Bar` or alternatively it could be the assignment of one OID-IRI value containing a multi-line comment to `Foo`.
    ```asn1
    Foo MyType ::= "/a /*comment"
    Bar MyType ::= "*//b"
    ```
    This ambiguity cannot be resolved until MyType is resolved to either a character string type or an OID-IRI.
    Therefore, this parser has taken the decision to always parse the above example as two assignments of character string literals and never as an OID-IRI value containing a multi-line comment.

# Unicode
This program aims to comply with the requirements within the Unicode Standard, Version 15.1.0.
- The only accepted source code input encoding is UTF-8.  Any other encoding should be converted to UTF-8 prior to processing with this program.
- Any reference to characters within this program and its documentation should be taken to mean Unicode Code Point.
- If the first character of a source file is the byte order mark (U+FEFF) then it will be ignored.  It will not be ignored in any other locations within a source file.

This program aims to comply with the requirements for identifiers, as specified in Version 15.1.0 of the Unicode Standard. See Unicode Standard Annex #31, “Unicode Identifiers and Syntax” (https://www.unicode.org/reports/tr31/tr31-39.html).
- UAX31-R1 Default Identifiers:  Complied with, using a profile as follows:
    - Start := XID_Start plus the characters $ (U+0024) and _ (U+005F)
    - Continue := XID_Continue plus the characters Hyphen (U+2010) and Hyphen-Minus (U+002D)
    - Medial := empty

    With the following exceptions:
    - No identifiers can end with a hyphen character.
    - No identifiers can contain two consecutive hyphens.
- UAX31-R1b Stable Identifiers:  Not complied with.  No stability guarantees are made beyond that which are made in UAX31-R1.
- UAX31-R2 Immutable Identifiers: Not complied with for the same reasons as UAX31-R1b is not complied with.
- UAX31-R3 Pattern_White_Space and Pattern_Syntax Characters: Not currently complied with, but might be complied with in the future.
- UAX31-R4 Equivalent Normalized Identifiers: Complied with, using a profile as follows:
    - NFC normalisation for all identifiers.
    - All Hyphen (U+2010) and Hyphen-Minus (U+002D) characters will be treated as equivalent in identifiers.
    - Case detection of the first character shall check for General_Category=Uppercase_Letter to see if a character begins with an uppercase letter.  All other characters, including non-cased characters are treated as lowercase for identifier parsing.
    - Case detection of all uppercase identifiers shall exclude all characters with General_Category=Lowercase_Letter and allow all other characters.
    - Note that this case detection is not guaranteed to be stable between Unicode versions, however likely will be for commonly used scripts and characters.
- UAX31-R5 Equivalent Case-Insensitive Identifiers: Not complied with as case insensitive identifiers are not used.
- UAX31-R6 Filtered Normalized Identifiers: Not complied with as no filtering is performed.
- UAX31-R7 Filtered Case-Insensitive Identifiers: Not complied with as no filtering is performed.
- UAX31-R8 Extended Hashtag Identifiers: Not complied with as hashtags are not relevant.

UTS#55, UTS#39 and UTR#36 are not currently complied with.
