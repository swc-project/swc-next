# Function to clone a repository in the background
clone_repo() {
    local repo="$1"
    local path="$2"
    local ref="$3"
    local name="$4"

    echo "Starting clone of $name..."
    (
        # Create the directory structure if it doesn't exist
        mkdir -p "$(dirname "$path")"

        # Clone the repository with minimal progress output
        git clone --quiet --no-progress --single-branch --depth 1 \
            "https://github.com/$repo.git" "$path"

        # Checkout the specific commit
        cd "$path"
        git fetch --quiet --depth 1 origin "$ref"
        git checkout --quiet "$ref"

        echo "✓ Completed clone of $name"
    )
}

clone_repo "tc39/test262" "tasks/testsuite/fixtures/test262" "079b13f85da120049919365612163e4579678e89" "test262"
clone_repo "tc39/test262-parser-tests" "tasks/testsuite/fixtures/test262-parser-tests" "0e808c74fbec780646434cad17bb22dc52461003" "test262-parser"
