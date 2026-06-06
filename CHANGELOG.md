# Changelog

All notable changes to this project will be documented in this file.

## 0.1.12 (2026-06-07)

* security fix: `rustls-webpki` updated to prevent GHSA-82j2-j2ch-gfr8
* chore: updated to Rust v1.96
* chore: updated dependencies
* fix: added database readiness check for CI

## 0.1.11 (2026-03-23)

* security fix: `aws-lc-sys` updated to prevent GHSA-394x-vwmw-crm3
* chore: updated dependencies

## 0.1.10 (2026-03-10)

* security fix: `aws-lc-sys` updated to prevent GHSA-vw5v-4f2q-w9xf, GHSA-65p9-r9h6-22vj, GHSA-hfpc-8r3f-gw53
* chore: updated dependencies
* chore: updated to Rust v1.94

## 0.1.9 (2026-02-11)

* security fix: `jsonwebtoken` updated to 0.10.3 to prevent CVE-2026-25537
* security fix: `bytes` updated to 1.11.1 to prevent CVE-2026-25541
* chore: updated other dependencies
* chore: updated to Rust v1.93

## 0.1.8 (2026-01-13)

* security fix: `rsa` updated to 0.9.10 to prevent CVE-2026-21895
* chore: updated to axum 0.8.8
* chore: updated other dependencies

## 0.1.7 (2025-12-14)

* chore: updated to axum 0.8.7 and Rust v1.92
* chore: updated other dependencies
* security fix: using `aws_lc_rs` crypto backend for `jsonwebtoken` to prevent RUSTSEC-2023-0071

## 0.1.6 (2025-09-22)

* refactor: upgrade to Rust v1.90
* chore: updated dependencies

## 0.1.5 (2025-09-17)

* security fix: tracing-subscriber upgraded to prevent CVE-2025-58160
* refactor: upgrade to Rust v1.89
* chore: updated dependencies

## 0.1.4 (2025-04-09)

* chore: updated dependencies

## 0.1.3 (2025-02-20)

* refactor: upgrade to Rust v1.85, edition 2024
* refactor: error handling
* docs: updated readme, changelog and API documentation  

## 0.1.2 (2025-01-29)

* feat: transaction samples.
* feat: structured error response.

## 0.1.1 (2025-01-04)

* fix: breaking changes after upgrading to axum 0.8

## 0.1.0 (2023-11-09)

* project started.
