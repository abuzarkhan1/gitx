use crate::connection::Connection;
use crate::error::Result;
use crate::models::*;
use rusqlite::params;

pub struct RepositoryStore<'a> {
    conn: &'a mut Connection,
}

impl<'a> RepositoryStore<'a> {
    pub fn new(conn: &'a mut Connection) -> Self {
        Self { conn }
    }

    pub fn insert_repository(&mut self, root_path: &str, git_dir: &str) -> Result<i64> {
        let tx = self.conn.transaction()?;
        let now = "now"; // Or actual timestamp
        tx.execute(
            "INSERT INTO repositories (root_path, git_dir, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![root_path, git_dir, now, now],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    pub fn insert_author(&mut self, name: &str, email: Option<&str>) -> Result<i64> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO authors (name, email) VALUES (?1, ?2)",
            params![name, email],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    pub fn insert_commits_batch(&mut self, commits: &[Commit]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO commits (oid, author_id, committer_id, tree_oid, timestamp, message) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            )?;
            for commit in commits {
                stmt.execute(params![
                    commit.oid,
                    commit.author_id,
                    commit.committer_id,
                    commit.tree_oid,
                    commit.timestamp,
                    commit.message
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_commit_parents_batch(&mut self, parents: &[CommitParent]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO commit_parents (commit_oid, parent_oid, parent_index) VALUES (?1, ?2, ?3)"
            )?;
            for parent in parents {
                stmt.execute(params![parent.commit_oid, parent.parent_oid, parent.parent_index])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_files_batch(&mut self, files: &[File]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO files (path, first_commit_oid, last_commit_oid, language, is_current) VALUES (?1, ?2, ?3, ?4, ?5)"
            )?;
            for file in files {
                stmt.execute(params![
                    file.path,
                    file.first_commit_oid,
                    file.last_commit_oid,
                    file.language,
                    file.is_current
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_file_changes_batch(&mut self, changes: &[FileChange]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO file_changes (commit_oid, file_id, change_type, old_path, new_path, insertions, deletions) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            )?;
            for change in changes {
                stmt.execute(params![
                    change.commit_oid,
                    change.file_id,
                    change.change_type,
                    change.old_path,
                    change.new_path,
                    change.insertions,
                    change.deletions
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_branches_batch(&mut self, branches: &[Branch]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO branches (name, tip_oid, is_remote, is_default) VALUES (?1, ?2, ?3, ?4)"
            )?;
            for branch in branches {
                stmt.execute(params![
                    branch.name,
                    branch.tip_oid,
                    branch.is_remote,
                    branch.is_default
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn insert_tags_batch(&mut self, tags: &[Tag]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO tags (name, target_oid) VALUES (?1, ?2)"
            )?;
            for tag in tags {
                stmt.execute(params![
                    tag.name,
                    tag.target_oid
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}
