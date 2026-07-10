# shellcheck shell=bash
# Suite-level setup: severs host git-config and keyring inheritance so every
# fixture behaves identically on any host.
# (BATS auto-loads this for directory-level runs; bash per ADR-0020.)

setup_suite() {
    # gpg refuses a group/world-accessible homedir, and BATS_SUITE_TMPDIR can
    # inherit 0755 from the host umask; lock the sandbox to 0700 BEFORE
    # exporting GNUPGHOME at it.
    local gnupghome="$BATS_SUITE_TMPDIR/gnupg"
    mkdir -p "$gnupghome"
    chmod 700 "$gnupghome"
    export GNUPGHOME="$gnupghome"

    # Suite-owned global git config: neutral fixture identity, main as the
    # default branch, signing off. Written with `git config --file` so git owns
    # the format.
    local gitconfig="$BATS_SUITE_TMPDIR/gitconfig"
    git config --file "$gitconfig" user.name "Bats Fixture"
    git config --file "$gitconfig" user.email "bats-fixture@example.com"
    git config --file "$gitconfig" init.defaultBranch main
    git config --file "$gitconfig" commit.gpgsign false
    git config --file "$gitconfig" tag.gpgsign false

    export GIT_CONFIG_GLOBAL="$gitconfig"
    export GIT_CONFIG_SYSTEM=/dev/null
}
