#!/usr/bin/env bash
# Generates OUR OWN throwaway RSA-1024 key and self-signed certificate for the local,
# offline portal (cligate) TLS server in gw2-login. This is not ArenaNet's key: the
# client's pinned cligate.101 modulus gets replaced in process memory only (see
# docs/re/209026-portal-pins.md).
#
# Output goes to data/tls/ (gitignored via /data/). The private key is never printed.
#
#   tools/gen-test-cert.sh [--force] [OUT_DIR]
#   SUBJ='/C=US/.../CN=host' DNS_NAME=host tools/gen-test-cert.sh
#
# The default subject is byte-identical to the Name pinned beside the cligate.101 key
# in build 209026 (.rdata 0x14230b450..0x14230b4aa): C, ST, L, CN as PrintableString,
# no O. The validator matches issuer Name contents against its trust table and
# requires a DNS SAN and SHA-1/RSA certificate signature. See
# docs/re/209026-certificate-check.md. These legacy settings are for this offline
# build's test identity only; certificate compatibility does not configure trust.
set -euo pipefail

force=0
if [[ "${1:-}" == "--force" ]]; then force=1; shift; fi
root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$root/data/tls}"
subj="${SUBJ:-/C=US/ST=Washington/L=Bellevue/CN=cligate.101.ncplatform.net}"
# The runtime TLS hostname includes its DNS root dot; the issuer CN does not.
# REA run 39 confirms exact dotted SAN acceptance (docs/re/209026-login-tls.md).
dns_name="${DNS_NAME:-cligate.101.ncplatform.net.}"
key="$out/cligate-101.key.pem"
cert="$out/cligate-101.cert.der"

abs="$(realpath -m "$out")"
if [[ "$abs/" == "$root/"* ]] && ! git -C "$root" check-ignore -q "$abs/cligate-101.key.pem"; then
  echo "refusing: $abs is inside the repo and not gitignored" >&2; exit 2
fi
mkdir -p "$out"
if [[ -e "$key" && $force -eq 0 ]]; then
  echo "exists: $key (use --force to replace)" >&2; exit 1
fi

umask 077
cfg="$(mktemp)"; trap 'rm -f "$cfg"' EXIT
# string_mask=default picks PrintableString for printable values, matching the pinned Name.
printf '[req]\ndistinguished_name=dn\nstring_mask=default\n[dn]\n' > "$cfg"
openssl req -config "$cfg" -x509 -newkey rsa:1024 -nodes -days 3650 -sha1 \
  -subj "$subj" -addext "subjectAltName=DNS:$dns_name" \
  -addext 'keyUsage=critical,keyEncipherment' \
  -keyout "$key" -outform DER -out "$cert" 2>/dev/null
chmod 600 "$key"

# Checks: 1024-bit modulus (exactly 128 bytes, top bit set) and e = 65537.
bits="$(openssl rsa -in "$key" -noout -text 2>/dev/null | sed -n 's/^Private-Key: (\([0-9]*\) bit.*/\1/p')"
exp="$(openssl x509 -inform DER -in "$cert" -noout -text | sed -n 's/^ *Exponent: \([0-9]*\).*/\1/p')"
mod="$(openssl x509 -inform DER -in "$cert" -noout -modulus | cut -d= -f2)"
[[ "$bits" == 1024 && "$exp" == 65537 && ${#mod} -eq 256 && "${mod:0:1}" =~ [89A-F] ]] \
  || { echo "unexpected key shape: bits=$bits e=$exp modhex=${#mod}" >&2; exit 3; }

echo "key:  $key (private, 0600, not printed)"
echo "cert: $cert"
echo "subject: $(openssl x509 -inform DER -in "$cert" -noout -subject -nameopt RFC2253)"
echo "modulus sha256: $(printf '%s' "$mod" | xxd -r -p | sha256sum | cut -c1-16)"
