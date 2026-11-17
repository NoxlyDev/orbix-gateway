#!/usr/bin/env bash
# Jalankan dari root orbix-gateway setelah unzip patch ini di atasnya
rm -f \
  crates/orbix_connectors/src/connectors/orbixthreedsserver.rs \
  crates/orbix_connectors/src/connectors/orbixthreedsserver/transformers.rs \
  crates/router/tests/connectors/orbixthreedsserver.rs \

rm -f apply_patch.sh
echo "patch applied"
