use std::io::{Read, Write};

use age::{
    armor::{ArmoredWriter, Format},
    decryptor::{self, RecipientsDecryptor},
    encrypted,
    x25519::Identity,
    Decryptor, Encryptor, Recipient,
};
use clap::Id;
use serde::{Deserialize, Serialize};

use eyre::{ContextCompat, Result};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EncInput {
    pub(crate) name: String,
    pub(crate) webhook: String,
}

pub(crate) fn encrypt(
    input: EncInput,
    recipients: Vec<Box<dyn Recipient + Send>>,
) -> Result<String> {
    let encryptor = Encryptor::with_recipients(recipients).wrap_err("Failed to build encryptor")?;
    let mut encrypted = vec![];
    let mut writer = encryptor.wrap_output(&mut encrypted)?;

    let inp_bytes = postcard::to_stdvec(&input)?;
    writer.write_all(&inp_bytes)?;
    writer.finish()?;

    let encrypted_encoded = data_encoding::BASE64URL.encode(&encrypted);

    Ok(encrypted_encoded)
}

pub(crate) fn decrypt(input: String, id: &Identity) -> Result<EncInput> {
    let decoded = data_encoding::BASE64URL.decode(input.as_bytes())?;

    let decryptor = match age::Decryptor::new(&decoded[..])? {
        Decryptor::Recipients(d) => d,
        Decryptor::Passphrase(_) => panic!(),
    };

    let mut decrypted = vec![];
    let mut reader = decryptor.decrypt(std::iter::once(id as &dyn age::Identity))?;
    reader.read_to_end(&mut decrypted)?;

    let parsed = postcard::from_bytes(&decrypted)?;

    Ok(parsed)
}

#[test]
fn encrypt_decrypt() {
    let subject = EncInput {
        name: String::from("name"),
        webhook: String::from("webhook"),
    };

    let id = age::x25519::Identity::generate();

    let encrypted = encrypt(subject.clone(), vec![Box::new(id.to_public())]).unwrap();

    let subject2 = decrypt(encrypted, &id).unwrap();

    assert_eq!(subject, subject2);
}
