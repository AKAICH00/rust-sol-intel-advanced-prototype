# Making a New Release

To create and publish a new release of the crate to crates.io:

## 1. Update the Version

- Update the version in `jup-ag-sdk/Cargo.toml` following [semantic versioning](https://semver.org/), e.g., `1.0.2` → `1.0.3` for a patch update.

## 2. Create a Pull Request

- Create a branch for the version update:
  ```sh
  git checkout -b bump-version-x.y.z
  git add jup-ag-sdk/Cargo.toml
  git commit -m "Bump version to vX.Y.Z"
  git push origin bump-version-x.y.z
  ```
- Create a pull request to merge these changes into the main branch
- Get the PR reviewed and approved
- Once merged, the auto-tag workflow will run automatically

## 3. Automatic Tag Creation

- The GitHub Actions workflow will automatically create a tag matching the version in Cargo.toml when the PR is merged
- You can check the status of the auto-tag workflow in the [Actions tab](https://github.com/Jupiter-DevRel/jup-rust-sdk/actions/workflows/auto-tag.yml)

## 4. Run the Publish Workflow

- Go to the [Publish Workflow page](https://github.com/Jupiter-DevRel/jup-rust-sdk/actions/workflows/publish.yml)
- Click on "Run workflow" button (dropdown on the right side)
- Select the branch (usually "main") and click "Run workflow"
- The workflow will:
    - Verify that the version in the tag matches the version in Cargo.toml
    - Build the package using Cargo
    - Publish the crate to crates.io

## 5. Confirm the Release

- Check [crates.io](https://crates.io/crates/jup-ag-sdk) to ensure the new version has been published successfully.

## Troubleshooting

- If the automated publishing fails, check the GitHub Actions logs for details
- Ensure your CRATES_IO_TOKEN is properly set in the repository secrets
- For manual publishing (if needed):
  ```sh
  cargo login
  cargo publish -p jup-ag-sdk/ --dry-run  # Test run first
  cargo publish -p jup-ag-sdk/            # Actual publish
  ```