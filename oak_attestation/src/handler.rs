//
// Copyright 2022 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use alloc::{sync::Arc, vec::Vec};
use core::future::Future;

use anyhow::Context;
use oak_crypto::{
    encryption_key::{AsyncEncryptionKeyHandle, EncryptionKeyHandle},
    encryptor::ServerEncryptor,
    proto::oak::crypto::v1::{EncryptedRequest, EncryptedResponse},
    EMPTY_ASSOCIATED_DATA,
};

/// Information about a public key.
#[derive(Debug, Clone)]
pub struct PublicKeyInfo {
    /// The serialized public key.
    pub public_key: Vec<u8>,
    /// The serialized attestation report that binds the public key to the specific version of the
    /// code running in a TEE.
    pub attestation: Vec<u8>,
}

/// Wraps a closure to an underlying function with request encryption and response decryption logic,
/// based on the provided encryption key.
pub struct EncryptionHandler<H: FnOnce(Vec<u8>) -> Vec<u8>> {
    // TODO(#3442): Use attester to attest to the public key.
    encryption_key_handle: Arc<dyn EncryptionKeyHandle>,
    request_handler: H,
}

impl<H: FnOnce(Vec<u8>) -> Vec<u8>> EncryptionHandler<H> {
    pub fn create(encryption_key_handle: Arc<dyn EncryptionKeyHandle>, request_handler: H) -> Self {
        Self {
            encryption_key_handle,
            request_handler,
        }
    }
}

impl<H: FnOnce(Vec<u8>) -> Vec<u8>> EncryptionHandler<H> {
    pub fn invoke(self, encrypted_request: &EncryptedRequest) -> anyhow::Result<EncryptedResponse> {
        // Decrypt request.
        let (server_encryptor, request, _) =
            ServerEncryptor::decrypt(encrypted_request, self.encryption_key_handle.as_ref())
                .context("couldn't create server encryptor")?;

        // Handle request.
        let response = (self.request_handler)(request);

        // Encrypt and serialize response.
        // The resulting decryptor for subsequent requests is discarded because we don't expect
        // another message from the stream.
        server_encryptor
            .encrypt(&response, EMPTY_ASSOCIATED_DATA)
            .context("couldn't encrypt response")
    }
}

/// Wraps a closure to an underlying function with request encryption and response decryption logic,
/// based on the provided encryption key.
/// [`AsyncEncryptionHandler`] can be used when an [`AsyncEncryptionKeyHandle`] is needed.
pub struct AsyncEncryptionHandler<G, H, F>
where
    G: AsyncEncryptionKeyHandle + Send + Sync,
    H: FnOnce(Vec<u8>) -> F,
    F: Future<Output = Vec<u8>>,
{
    // TODO(#3442): Use attester to attest to the public key.
    encryption_key_handle: Arc<G>,
    request_handler: H,
}

impl<G, H, F> AsyncEncryptionHandler<G, H, F>
where
    G: AsyncEncryptionKeyHandle + Send + Sync,
    H: FnOnce(Vec<u8>) -> F,
    F: Future<Output = Vec<u8>>,
{
    pub fn create(encryption_key_handle: Arc<G>, request_handler: H) -> Self {
        Self {
            encryption_key_handle,
            request_handler,
        }
    }

    pub async fn invoke(
        self,
        encrypted_request: &EncryptedRequest,
    ) -> anyhow::Result<EncryptedResponse> {
        // Decrypt request.
        let (server_encryptor, request, _associated_data) =
            ServerEncryptor::decrypt_async(encrypted_request, self.encryption_key_handle.as_ref())
                .await
                .context("couldn't decrypt request")?;

        // Handle request.
        let response = (self.request_handler)(request).await;

        // Encrypt and serialize response.
        // The resulting decryptor for consequent requests is discarded because we don't expect
        // another message from the stream.
        server_encryptor
            .encrypt(&response, EMPTY_ASSOCIATED_DATA)
            .context("couldn't encrypt response")
    }
}
