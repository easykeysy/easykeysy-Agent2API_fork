#!/usr/bin/env bash
# ： by publish-installers.yml 。
# onlyused for old or unavailable when 。
#
# bash scripts/release.sh vX.Y.Z # this tag available run
# bash scripts/release.sh vX.Y.Z <run-id> # or run
#
# ── 2026-10： and in ────────────────────────
# already to `Agent2Builder/agent2api-releases`（ tag 1 
# `repository_dispatch`， `.github/workflows/build.yml`）。
# run、 down artifact、 Release to ****， not is currentdirectory 
# origin。available `BUILD_REPO` ：
#
#   BUILD_REPO=SomeOwner/some-repo bash scripts/release.sh v3.2.0
#
# not 「current origin」： in ， run in 。
# this and `desktop-tauri/src-tauri/server/src/server/core/update/version.rs`
# `DEFAULT_REPO`、 and `app.json` `app.repo` must to 1 
# —— not 1 is 「Release ， no has to update」。
#
# ：
# 1. down artifact（windows-nsis / macos-universal /
# linux-desktop / android-apk / ios-ipa， and linux-server-amd64 / -arm64
# update） to dist/；
# 2. use tag （= update， .agents/skills/release-flow/SKILL.md
# 5 ）create / update GitHub Release and ；
# 3. 。
#
# ── artifact and new tag ──────────────────────────────────
# artifact is up ， tag run can can ：
# · linux-server-*（ 1 update，v3.0.4 ）
# · linux-desktop（Linux deb / AppImage，v3.0.9 ）
# · android-apk（ APK）
# · ios-ipa（ iOS ， / already can can ）
# 1 when **、 not in ** —— tag can 。
#
# etc can ： `--clobber` up 。

set -euo pipefail

TAG="${1:?用法: scripts/release.sh vX.Y.Z [build-run-id]}"
RUN_ID="${2:-}"

# run and Release in （file）。default and `version.rs` 
# `DEFAULT_REPO` 1 ； when 1 。
BUILD_REPO="${BUILD_REPO:-Agent2Builder/agent2api-releases}"
export BUILD_REPO

# ── 、 ────────────────────────────────────────
# use ， —— Release can 
# by ** has that 1 **create / 。 use 1 `GH_TOKEN`， use 
# token edit Release， `HTTP 404: Not Found`
# （GitHub for none 404 not is 403，「Release not in 」）。
#
# `BUILD_TOKEN`： for has contents:write token（defaultcurrent gh ）
# `SOURCE_TOKEN`： for has contents:write token（ up ）
#
# when ，current `gh auth` for has —— 1 
# （fork、） is this ， not 。
BUILD_TOKEN="${BUILD_TOKEN:-${GH_TOKEN:-}}"
SOURCE_TOKEN="${SOURCE_TOKEN:-${GH_TOKEN:-}}"

case "$TAG" in
  v*) ;;
  *) echo "错误: '$TAG' 不是 v* 形式的 tag" >&2; exit 1 ;;
esac

# ── beta ：tag after （v3.1.0-beta.1） when ，Release must pre-release ──
# this is use 「 beta 」 ： GitHub 
# /releases/latest，**excluding** ； has from /releases
# beta 。 = beta will all use update to ；
# to then not update。 this by tag 、 and already in Release。
case "${TAG#v}" in
  *-*) PRERELEASE=true ;;
  *)   PRERELEASE=false ;;
esac
if [ "$PRERELEASE" = true ]; then
  echo "→ 检测到预发布后缀（${TAG#v}），Release 标记为 pre-release（只发给「加入 beta 版」的用户）"
fi

command -v gh   >/dev/null || { echo "错误: 未安装 gh CLI" >&2; exit 1; }
# gh JSON use node （ not jq ， has node）
command -v node >/dev/null || { echo "错误: 未安装 node" >&2; exit 1; }

# gh is Go ： not git http.proxy， not Windows ，
# 。 git gh no ， down down and up is file
# ， not when will in failure —— git to gh。
# HTTPS_PROXY：gh request（ up down ） is HTTPS， not HTTP_PROXY。
# from git ， git 1 ；git not ，
# （ already when not ）。
if [ -z "${HTTPS_PROXY:-}" ]; then
  # by git/curl ， this config：
  # http.<url>.proxy（ by ） / http.proxy / https.proxy
  # --get-urlmatch will https.proxy， https.proxy will before 。
  GH_PROXY=$(git config --get-urlmatch http.proxy https://github.com || true)
  [ -n "$GH_PROXY" ] || GH_PROXY=$(git config --get https.proxy || true)
  if [ -n "$GH_PROXY" ]; then
    export HTTPS_PROXY="$GH_PROXY"
    echo "→ 已启用代理: $GH_PROXY"
  fi
fi

# ── 1. build run and down ────────────────────────────
if [ -z "$RUN_ID" ]; then
  echo "→ 查找 $TAG 触发的 build 工作流…"
  # not by run conclusion ：「 artifact is 」，
  # successfailure run old available
  for id in $(GH_TOKEN="$BUILD_TOKEN" gh run list --workflow=release --repo "$BUILD_REPO" --limit 30 --json databaseId,headBranch \
      | node -e 'let s="";process.stdin.on("data",d=>s+=d);process.stdin.on("end",()=>{JSON.parse(s).filter(r=>r.headBranch===process.argv[1]).forEach(r=>process.stdout.write(r.databaseId+"\n"))})' "$TAG"); do
    if GH_TOKEN="$BUILD_TOKEN" gh api "repos/$BUILD_REPO/actions/runs/$id/artifacts" \
         --jq 'any(.artifacts[].name; . == "macos-universal") and any(.artifacts[].name; . == "windows-nsis")' >/dev/null 2>&1; then
      RUN_ID=$id
      break
    fi
  done
  [ -n "$RUN_ID" ] || {
    echo "错误: 没找到 $TAG 的可用 build run（需 macos-universal 与 windows-nsis 两个 artifact 齐全）—— 先确认 CI 跑完（gh run list），或手动传 run-id" >&2
    exit 1
  }
fi
echo "→ build run: $RUN_ID"

rm -rf dist
GH_TOKEN="$BUILD_TOKEN" gh run download "$RUN_ID" --repo "$BUILD_REPO" -D dist
EXE=$(ls dist/windows-nsis/*.exe)
DMG=$(ls dist/macos-universal/*.dmg)
echo "→ already down $(basename "$EXE") / $(basename "$DMG")"

ASSETS=("$EXE" "$DMG")

# Linux ：deb and AppImage（build linux-desktop job up ）。
# this 1 for is can up after has artifact —— tag run no has ， 1 
# （ and down linux-server 1 ： tag can ）。
#
# ⚠️ use find -maxdepth 2 not is `dist/linux-desktop/*` 1 ： up 
# path is `target/release/bundle/deb/*.deb` and `.../appimage/*.AppImage`，
# upload-artifact ****（`target/release/bundle`） as ，
# is is `linux-desktop/deb/…` and `linux-desktop/appimage/…` ——
# 1 not to （2026-10-17 ：Release ， no has deb/AppImage
# ）。 its artifact path is `dist/*`， dist， is 。
if [ -d dist/linux-desktop ]; then
  LINUX_PKGS=()
  while IFS= read -r file; do
    LINUX_PKGS+=("$file")
  done < <(find dist/linux-desktop -maxdepth 2 -type f \( -name '*.deb' -o -name '*.AppImage' \) | sort)
  if [ ${#LINUX_PKGS[@]} -gt 0 ]; then
    ASSETS+=("${LINUX_PKGS[@]}")
    # use ${arr[*]##*/} not is $(basename "${arr[@]}")： Git Bash 
    # /usr/bin/basename 1 （POSIX ）， will after 
    # —— AppImage when 「deb no down to 」（2026-10-17 ）
    echo "→ 已下载 Linux 桌面端包: ${LINUX_PKGS[*]##*/}"
  else
    echo "⚠️  linux-desktop artifact 里没有 .deb / .AppImage —— Linux 用户将看不到可下载的安装包"
  fi
else
  echo "⚠️  这个 run 里没有 linux-desktop 产物（Linux 桌面端上线前的 tag？）——"
  echo "    Release 将不含 Linux 桌面安装包，Linux 面板的「下载并安装」会被藏起"
fi

# update / ： tar.gz + .sha256（ by 
# file， will ）。 tag run no has this artifact —— 1 。
#
# 1 ： 1 update + Linux （
# install.sh / systemd / env ， after install.sh can ）。
if [ -d dist/linux-server-amd64 ] && [ -d dist/linux-server-arm64 ]; then
  # directory down is <>.tar.gz and <>.tar.gz.sha256， up 
  for file in dist/linux-server-amd64/* dist/linux-server-arm64/*; do
    ASSETS+=("$file")
  done
  echo "→ already down $(basename "$(ls dist/linux-server-amd64/*.tar.gz)") / $(basename "$(ls dist/linux-server-arm64/*.tar.gz)")（ .sha256）"
else
  echo "⚠️ this run no has linux-server-* （ can up before tag？）——"
  echo " Release will not update，update will 「」"
fi

# Windows server（none GUI， + Windows ）。
# and up linux-server 1 ： tag run no has ， 1 。
if [ -d dist/windows-server-amd64 ]; then
  for file in dist/windows-server-amd64/*; do
    ASSETS+=("$file")
  done
  echo "→ already down $(basename "$(ls dist/windows-server-amd64/*.zip)")（ .sha256）"
else
  echo "⚠️ this run no has windows-server-amd64 （ can up before tag？）——"
  echo " Release will not Windows server，Windows Server use can "
fi

# ── ：default** not ** Release ────────────────────────
# `INCLUDE_MOBILE=1` APK / IPA up 。
#
# Whydefault： in （2026-10 use need not ）。
# artifact， need need when in Actions down ； is not to Release up ——
# up etc for 「 this can use 」， in not can 。
#
# when ：`INCLUDE_MOBILE=1 bash scripts/release.sh vX.Y.Z`， or default
# 1。 APK / IPA need 。
INCLUDE_MOBILE="${INCLUDE_MOBILE:-0}"

if [ "$INCLUDE_MOBILE" = "1" ]; then
  # Android APK（build android job up ， .sha256）。
  # CI before tag no has this artifact —— 1 。
  #
  # and before not ，APK ** is use to file**： is 
  # `Agent2API-<>.apk`，android job already by this ， this up can 。
  if [ -d dist/android-apk ]; then
    for file in dist/android-apk/*; do
      ASSETS+=("$file")
    done
    echo "→ already down $(basename "$(ls dist/android-apk/*.apk)")（ .sha256）"
  else
    echo "⚠️ this run no has android-apk （APK CI before tag？）——"
    echo " Release will not ，Android use can from old or get"
  fi

  # iOS IPA（build ios job up ， .sha256）。
  # default is ****： use AltStore / SideStore use Apple ID after ；
  # iOS secret is already 。 can Release， not 。
  if [ -d dist/ios-ipa ]; then
    for file in dist/ios-ipa/*; do
      ASSETS+=("$file")
    done
    echo "→ already down $(basename "$(ls dist/ios-ipa/*.ipa)")（ .sha256）"
  else
    echo "⚠️ this run no has ios-ipa （iOS CI before tag？）——"
    echo " Release will not iOS ，iPhone use can PWA or "
  fi
else
  echo "→ by set not Release（INCLUDE_MOBILE=0）； in Actions artifact "
fi

# ── 2. update = tag ──────────────────────
NOTES_FILE=$(mktemp)
trap 'rm -f "$NOTES_FILE"' EXIT
git log -1 --format=%B "$TAG" > "$NOTES_FILE"

echo "→ GitHub Release"
if GH_TOKEN="$BUILD_TOKEN" gh release view "$TAG" --repo "$BUILD_REPO" >/dev/null 2>&1; then
  # already in when 1 （ / this 1 ）
  GH_TOKEN="$BUILD_TOKEN" gh release edit "$TAG" --repo "$BUILD_REPO" --prerelease="$PRERELEASE"
  echo " ↳ already in ， up （pre-release=$PRERELEASE）"
else
  GH_TOKEN="$BUILD_TOKEN" gh release create "$TAG" --repo "$BUILD_REPO" --prerelease="$PRERELEASE" --title "$TAG" \
    --notes-file "$NOTES_FILE"
  echo " ↳ already create（pre-release=$PRERELEASE）"
fi

# ** ** up ， not 1 1 。（2026-10-17，v3.0.9-beta.1）： 1 10 
# when gh will in 「 already in 」 up HTTP 422 ReleaseAsset.name already
# exists（--clobber no can ）， in —— is Release 、
# 1 。 + --clobber ， and etc （ that ）。
for asset in "${ASSETS[@]}"; do
  GH_TOKEN="$BUILD_TOKEN" gh release upload "$TAG" "$asset" --repo "$BUILD_REPO" --clobber
done

# ── 2.5 1 to （MIRROR_TO_SOURCE=1 when ）──────────────
# in ， use and Releases —— that 
# empty will as 「 this no 」。 1 。
#
# ； and publish-installers.yml。
# can ，none need down or new 。
#
# default；`MIRROR_TO_SOURCE=0` can （e.g. in 1 ）。
MIRROR_TO_SOURCE="${MIRROR_TO_SOURCE:-1}"
if [ "$MIRROR_TO_SOURCE" = "1" ]; then
  SOURCE_REPO="$(git remote get-url origin | sed -E 's#.*github\.com[:/]([^/]+/[^/.]+)(\.git)?$#\1#')"
  if [ -z "$SOURCE_REPO" ] || [ "$SOURCE_REPO" = "$BUILD_REPO" ]; then
    echo "⚠️ not to （origin not is GitHub？），"
  else
    echo "→ to $SOURCE_REPO"
    # and that ****（ 1 update）， not ——
    # Release 1 ， this 。
    if GH_TOKEN="$SOURCE_TOKEN" gh release view "$TAG" --repo "$SOURCE_REPO" >/dev/null 2>&1; then
      GH_TOKEN="$SOURCE_TOKEN" gh release edit "$TAG" --repo "$SOURCE_REPO" --prerelease="$PRERELEASE" --notes-file "$NOTES_FILE"
    else
      GH_TOKEN="$SOURCE_TOKEN" gh release create "$TAG" --repo "$SOURCE_REPO" --prerelease="$PRERELEASE" \
        --title "$TAG" --notes-file "$NOTES_FILE"
    fi
    # up ： 1 will in already in up 422（ up that ）
    for asset in "${ASSETS[@]}"; do
      GH_TOKEN="$SOURCE_TOKEN" gh release upload "$TAG" "$asset" --repo "$SOURCE_REPO" --clobber
    done
    echo " ↳ already ${#ASSETS[@]} "
  fi
fi

# ── 3. ───────────────────────────────────────────────
echo
echo "。 for ："
echo "  [1] GitHub Release:  gh release view $TAG --repo $BUILD_REPO"
if [ "$PRERELEASE" = true ]; then
  echo " ↑ beta ：Release Prerelease —— use not to ，"
  echo " has 「 beta 」 use to （ .agents/skills/release-flow/SKILL.md 6 ）"
else
  echo " [2] Docker Hub: https://hub.docker.com/r/aimodcc/agent2api/tags （$TAG + latest）"
fi
echo " [3] 「」 and $TAG 1 （Windows exe / macOS dmg / Linux deb+AppImage）"
echo " [4] Linux ：Release .deb and .AppImage in ； in Linux up deb（ or "
echo " AppImage） after ，「set → update」 can check to down 1 and to "
echo " [5] update：「set → update」「 down and update」， and Release "
echo " agent2api-server-linux-{amd64,arm64}.tar.gz and .sha256 in "
echo " [6] server（none GUI）：Release has windows-server-amd64.zip（ .sha256）。"
echo " after to —— Linux: install.sh / agent2api.service"
echo "      / agent2api.env.example；Windows: install-service.ps1"
echo " [7] ：Release has Agent2API-${TAG#v}.apk and .sha256； to Android up "
echo " can 、 can and （「 use 」， not ）"
echo " [8] iOS：Release has Agent2API-${TAG#v}.ipa and .sha256； need use "
echo " AltStore / SideStore after ， already can TestFlight / file"
if [ "$PRERELEASE" = true ]; then
  echo " [9] Docker ： $TAG that tag（beta tag not :latest）"
fi
