#!/usr/bin/env bash
# ex: set tabstop=8 softtabstop=0 expandtab shiftwidth=2 smarttab:

set -eu

SNAPCRAFT_BIN="${SNAPCRAFT:-snap run snapcraft}"
EXPORT_LOGIN="${EXPORT_LOGIN:-}"

if ! snap list snapcraft > /dev/null 2> /dev/null; then
  sudo snap install --candidate --classic snapcraft
fi


if ! ${SNAPCRAFT_BIN} whoami; then
  if [ -n "${EXPORT_LOGIN}" ]; then
    ${SNAPCRAFT_BIN} login --with - <<<${EXPORT_LOGIN}
  else
    echo "You need you set the EXPORT_LOGIN variable"
    exit 1
  fi 
fi


${SNAPCRAFT_BIN} remote-build --launchpad-accept-public-upload


git_branch="$(sed 's|^.*/||' .git/HEAD)"
if [ "${git_branch}" == 'master' ]; then
  release_channel="stable"
elif [ "${git_branch}" == 'dev' ]; then
  release_channel="candidate,beta,edge"
else
  echo "There is no release channel for this branch"
  exit 0
fi

for snap in *.snap; do
  ${SNAPCRAFT_BIN} upload "${snap}" --release "${release_channel}"
done


