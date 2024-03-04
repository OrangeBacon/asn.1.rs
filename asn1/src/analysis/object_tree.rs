//! Tools for dealing with Object Identifiers and Internationalized Resource
//! Identifiers (OIDs and IRIs)

/// An object identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Oid {
    /// Ordered list of all components in the OID
    pub components: Vec<OidComponent>,
}

/// A single component of an object identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OidComponent {
    /// the non-integer label for the component
    pub label: Option<String>,

    /// the integer label for the component.  note that this is not a number as
    /// math should not be done to it, it is an identifier.
    pub number: Option<String>,
}

/// An internationalized resource identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Iri {
    pub components: Vec<IriComponent>,
}

/// A single component of an iri.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IriComponent {
    pub label: String,
}

/// An error encountered while parsing an iri string
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IriParseError {
    MissingPrefix,
    MissingSuffix,
}

impl Iri {
    pub fn from_str(source: &str) -> Result<Self, IriParseError> {
        let mut iri = Iri { components: vec![] };

        let source = source
            .strip_prefix("\"/")
            .ok_or(IriParseError::MissingPrefix)?;
        let source = source
            .strip_suffix('\"')
            .ok_or(IriParseError::MissingSuffix)?;

        let components = source.split('/');
        for comp in components {
            let comp = comp.trim();
            iri.components.push(IriComponent { label: comp.to_string() });
        }

        Ok(iri)
    }
}
