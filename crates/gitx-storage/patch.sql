            CREATE INDEX idx_commits_author ON commits(author_id);
            CREATE INDEX idx_commits_committer ON commits(committer_id);
            CREATE INDEX idx_commit_parents_parent ON commit_parents(parent_oid);
            CREATE INDEX idx_file_changes_commit ON file_changes(commit_oid);
            CREATE INDEX idx_file_changes_file ON file_changes(file_id);
