use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::Saveable;
use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::{
    Signature as KSignature, SigningKey, VerifyingKey,
};
use rand_core::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::io::{
    Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature(pub KSignature);

#[derive(Clone, PartialEq, Eq)]
pub struct PublicKey(pub VerifyingKey);

#[derive(Clone)]
pub struct PrivateKey(pub SigningKey);

impl PrivateKey {
    pub fn new_key() -> Self {
        Self(SigningKey::random(&mut OsRng))
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(*self.0.verifying_key())
    }

    pub fn sign_hash(&self, hash: Hash) -> Signature {
        Signature(self.0.sign(&hash.bytes()))
    }
}

impl PublicKey {
    pub fn verify_hash(&self, hash: Hash, signature: &Signature) -> bool {
        self.0.verify(&hash.bytes(), &signature.0).is_ok()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_encoded_point(true).as_bytes().to_vec()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        VerifyingKey::from_sec1_bytes(bytes)
            .map(Self)
            .map_err(|e| BtcError::Crypto(e.to_string()))
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", hex::encode(self.to_bytes()))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_bytes())
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        KSignature::from_slice(&bytes)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

impl Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.to_bytes())
    }
}

impl<'de> Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        SigningKey::from_slice(&bytes)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl Saveable for PrivateKey {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to load private key"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to save private key"))
    }
}

impl Saveable for PublicKey {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to load public key"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "failed to save public key"))
    }
}
