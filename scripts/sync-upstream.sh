#!/usr/bin/env bash
# Pull upstream Era into the fork and replay the fork's commits on top.
#
#   main     mirror of upstream main; never commit to it
#   omarchy  main + the fork's commits (default branch on GitHub)
#
# Usage: scripts/sync-upstream.sh [--no-push]
#
# On a conflict the rebase stops: fix the files, `git add` them,
# `git rebase --continue`, then run this script again to test and push.
# rerere records each resolution, so the same conflict resolves itself
# next time.
#
# The whole body is one braced block: bash parses it before running, so the
# rebase can rewrite this file underneath it safely.
{
set -euo pipefail

UPSTREAM_URL=https://gitlab.gnome.org/TitouanReal/Era.git
FORK_BRANCH=omarchy
push=1
[[ ${1:-} == --no-push ]] && push=0

cd "$(git rev-parse --show-toplevel)"

git remote get-url upstream >/dev/null 2>&1 || git remote add upstream "$UPSTREAM_URL"
git config rerere.enabled true
git config rerere.autoupdate true

if [[ -d .git/rebase-merge || -d .git/rebase-apply ]]; then
    echo "A rebase is in progress. Finish it (git rebase --continue) or abort it first." >&2
    exit 1
fi
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Working tree has uncommitted changes; commit or stash them first." >&2
    exit 1
fi

git fetch --quiet upstream
git fetch --quiet origin 2>/dev/null || true

# main is a pure mirror, so it only ever fast-forwards.
old_main=$(git rev-parse main)
if ! git merge-base --is-ancestor "$old_main" upstream/main; then
    echo "main has commits upstream does not; it should be a pure mirror." >&2
    exit 1
fi
if [[ $(git branch --show-current) == main ]]; then
    git merge --quiet --ff-only upstream/main
else
    git branch -f main upstream/main
fi
new=$(git rev-list --count "$old_main..main")
echo "upstream: $new new commit(s)"
[[ $new -gt 0 ]] && git log --oneline "$old_main..main" | sed 's/^/  /'

git switch --quiet "$FORK_BRANCH"
if ! git merge-base --is-ancestor main "$FORK_BRANCH"; then
    # A safety net until the next successful sync.
    git branch -f "$FORK_BRANCH-before-sync" "$FORK_BRANCH"
    echo "rebasing $FORK_BRANCH onto main (backup: $FORK_BRANCH-before-sync)"
    if ! git rebase main; then
        cat >&2 <<'MSG'

Conflict. Resolve it, then:
    git add <files> && git rebase --continue
    scripts/sync-upstream.sh
Or give up with `git rebase --abort`. FORK.md lists what each hook is for.
MSG
        exit 1
    fi
fi

echo "fork commits on top of upstream:"
git log --oneline main.."$FORK_BRANCH" | sed 's/^/  /'

# The fork's own tests need the full build (libclepsydre), so they run only
# where that is available.
stage="$PWD/target/native/stage"
if [[ -f $stage/lib/pkgconfig/clepsydre-0.pc ]]; then
    export PKG_CONFIG_PATH="$stage/lib/pkgconfig" GI_TYPELIB_PATH="$stage/lib/girepository-1.0" LD_LIBRARY_PATH="$stage/lib"
    echo "running omarchy tests"
    cargo test --quiet --no-default-features --features backend-mock,platform-flatpak omarchy
else
    echo "skipping tests: run ./install.sh once first (see FORK.md)"
fi

if [[ $push == 1 ]] && git remote get-url origin >/dev/null 2>&1; then
    git push --quiet origin main
    git push --quiet --force-with-lease origin "$FORK_BRANCH"
    echo "pushed main and $FORK_BRANCH to origin"
fi
exit 0
}
