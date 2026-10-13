#!/usr/bin/env bash
# Run only on a disposable systemd host. Existing Terman packages are refused.
set -euo pipefail
[[ ${TERMAN_INSTALLER_TEST_ALLOW_HOST_CHANGES:-0} == 1 ]] || { echo 'Explicit host-change opt-in required' >&2; exit 1; }
[[ $(id -u) == 0 ]] || { echo 'Run elevated on an isolated test host' >&2; exit 1; }
previous=$(realpath "${1:?previous package directory}")
current=$(realpath "${2:?current package directory}")
format=${3:?deb or rpm}
case "$format" in deb|rpm) ;; *) exit 1 ;; esac
for tool in screen tmux htop; do
    if dpkg-query -W -f='${Status}' "terman-$tool" 2>/dev/null | grep -q 'install ok installed' ||
        rpm -q "terman-$tool" >/dev/null 2>&1; then
        echo 'Existing Terman packages: refusing to change prior state' >&2; exit 1
    fi
done
account="terman-test-$(od -An -N4 -tx4 /dev/urandom | tr -d ' ')"
work=$(mktemp -d "${TMPDIR:-/tmp}/terman-installer-test.XXXXXXXX")
uid=''
created=0
packages_installed=0
userctl() {
    runuser -u "$account" -- env XDG_RUNTIME_DIR="/run/user/$uid" \
        DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$uid/bus" systemctl --user "$@"
}
remove_packages() {
    if [[ $format == deb ]]; then dpkg --purge terman-screen terman-tmux terman-htop;
    else rpm -e terman-screen terman-tmux terman-htop; fi
}
cleanup() {
    if [[ $packages_installed == 1 ]]; then remove_packages || true; fi
    if [[ $created == 1 ]]; then
        userctl stop terman-screen.service terman-tmux.service || true
        loginctl disable-linger "$account" || true
        systemctl stop "user@$uid.service" || true
        userdel "$account" || true
    fi
    # Only this script's generated temp workspace may be removed recursively.
    [[ $work == "${TMPDIR:-/tmp}/terman-installer-test."* && -d $work ]] && rm -rf -- "$work"
}
trap cleanup EXIT
chmod 0755 "$work"
useradd --create-home --home-dir "$work/home" --shell /bin/bash "$account"
created=1
uid=$(id -u "$account")
loginctl enable-linger "$account"
systemctl start "user@$uid.service"
for attempt in {1..30}; do [[ -S /run/user/$uid/bus ]] && break; sleep 1; done
runuser -u "$account" -- mkdir -p "$work/home/.config/terman-screen" "$work/home/.config/terman-tmux"
for tool in screen tmux; do
    printf 'retained configuration\n' >"$work/home/.config/terman-$tool/installer-sentinel"
    chown "$account:$account" "$work/home/.config/terman-$tool/installer-sentinel"
done
install_packages() {
    local directory=$1
    local packages=()
    for tool in screen tmux htop; do
        mapfile -t candidates < <(find "$directory" -maxdepth 1 -name "terman-$tool*.$format" -type f)
        [[ ${#candidates[@]} == 1 ]] || { echo "Expected one $tool/$format package" >&2; exit 1; }
        packages+=("${candidates[0]}")
    done
    packages_installed=1
    if [[ $format == deb ]]; then dpkg -i "${packages[@]}";
    # Ubuntu's rpm database does not contain its distro-managed dependencies.
    else rpm -U --nodeps "${packages[@]}"; fi
}
install_packages "$previous"
for tool in screen tmux; do
    [[ -f /usr/lib/systemd/user/terman-$tool.service ]]
    [[ ! -f /usr/lib/systemd/system/terman-$tool.service ]]
    ! userctl is-active --quiet "terman-$tool.service"
    userctl enable --now "terman-$tool.service"
done
assert_services() {
    for tool in screen tmux; do
        userctl is-enabled --quiet "terman-$tool.service"
        userctl is-active --quiet "terman-$tool.service"
        pid=$(userctl show "terman-$tool.service" -p MainPID --value)
        [[ $pid != 0 && $(ps -o uid= -p "$pid" | tr -d ' ') == "$uid" ]]
        grep -q 'retained configuration' "$work/home/.config/terman-$tool/installer-sentinel"
    done
}
assert_services
runuser -u "$account" -- terman-screen -d -S installer-test --command 'sleep 60'
runuser -u "$account" -- terman-screen -S installer-test -X info
runuser -u "$account" -- terman-screen -S installer-test -X quit
runuser -u "$account" -- terman-tmux new-session -d -s installer-test 'sleep 60'
runuser -u "$account" -- terman-tmux list-sessions
runuser -u "$account" -- terman-tmux kill-session -t installer-test
# Upgrades must restart an active broker but leave an inactive one inactive.
userctl stop terman-tmux.service
install_packages "$current"
userctl is-active --quiet terman-screen.service
! userctl is-active --quiet terman-tmux.service
userctl start terman-tmux.service
assert_services
remove_packages
packages_installed=0
for tool in screen tmux; do
    ! userctl is-active --quiet "terman-$tool.service"
    [[ ! -e /usr/lib/systemd/user/terman-$tool.service ]]
    grep -q 'retained configuration' "$work/home/.config/terman-$tool/installer-sentinel"
done
echo "$format install, user service lifecycle, upgrade, removal and retention passed"
