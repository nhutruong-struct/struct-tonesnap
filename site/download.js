// Points the download buttons straight at the latest installer, so a visitor gets the file in one click
// instead of choosing among the release's assets. The buttons keep their link to the releases page as a
// fallback for when this script does not run or GitHub's API is unreachable or rate limited.
const LATEST_RELEASE = "https://api.github.com/repos/nhutruong-struct/struct-tonesnap/releases/latest";

async function latestInstallerUrl() {
  const response = await fetch(LATEST_RELEASE, { headers: { Accept: "application/vnd.github+json" } });
  if (!response.ok) throw new Error(`GitHub answered ${response.status}`);
  const release = await response.json();
  const installer = release.assets.find((asset) => asset.name.endsWith("-setup.exe"));
  if (!installer) throw new Error("the latest release has no installer");
  return installer.browser_download_url;
}

latestInstallerUrl()
  .then((url) => {
    for (const link of document.querySelectorAll("a[data-download]")) link.href = url;
  })
  .catch(() => {});
