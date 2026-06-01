use x25519_dalek::PublicKey as EncryptionPublicKey;

pub struct ZooProxyBuilderInfo {
    pub proxy_enc_public_key: EncryptionPublicKey,
}