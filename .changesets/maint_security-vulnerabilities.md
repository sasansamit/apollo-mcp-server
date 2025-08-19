### Address Security Vulnerabilities - @DaleSeo PR #264

This PR addresses the security vulnerabilities and dependency issues tracked in Dependency Dashboard #41 (https://osv.dev/vulnerability/RUSTSEC-2024-0388).

- Replaced the unmaintained `derivate` crate with the `educe` crate instead.
- Updated the `tantivy` crate.
