#!/bin/bash
sed -i '' 's/let mut r = r.map_err/let r = r.map_err/g' crates/gitx-git/src/repository.rs
sed -i '' 's/let mut head = self.repo.head()/let head = self.repo.head()/g' crates/gitx-git/src/repository.rs
sed -i '' 's/let mut delegate =/let delegate =/g' crates/gitx-git/src/diff.rs

# Swap name and target lines
sed -i '' 's/let target = r.into_fully_peeled_id().map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;/let NAME_TMP = r.name().shorten().to_string();\n            let target = r.into_fully_peeled_id().map_err(|e| GitError::Other(anyhow::anyhow!(e)))?;/g' crates/gitx-git/src/repository.rs
sed -i '' 's/let name = r.name().shorten().to_string();/let name = NAME_TMP;/g' crates/gitx-git/src/repository.rs

