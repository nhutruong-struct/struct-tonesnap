// Points the download buttons straight at the latest installer (Windows) or disk image (Mac), so a visitor
// gets the file in one click instead of choosing among the release's assets. The buttons keep their link to the releases page as a
// fallback for when this script does not run or GitHub's API is unreachable or rate limited.
const LATEST_RELEASE = "https://api.github.com/repos/nhutruong-struct/struct-tonesnap/releases/latest";

async function latestAssetUrls() {
  const response = await fetch(LATEST_RELEASE, { headers: { Accept: "application/vnd.github+json" } });
  if (!response.ok) throw new Error(`GitHub answered ${response.status}`);
  const release = await response.json();
  const url = (suffix) => release.assets.find((asset) => asset.name.endsWith(suffix))?.browser_download_url;
  return { windows: url("-setup.exe"), mac: url("_aarch64.dmg") };
}

latestAssetUrls()
  .then(({ windows, mac }) => {
    if (windows) for (const link of document.querySelectorAll("a[data-download]")) link.href = windows;
    if (mac) for (const link of document.querySelectorAll("a[data-download-mac]")) link.href = mac;
  })
  .catch(() => {});
