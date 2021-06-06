#!/bin/zsh
function _main() {
	_sync
	ssh -t "root@${SMARTOS_HOST}" gmake -C portunus $*
}

function _sync() {
	_check_smartos
	rsync --delete --exclude="target/*" -r . "root@${SMARTOS_HOST}:~/portunus"
}

function _check_smartos() {
	if [ -z "${SMARTOS_HOST}" ]; then
		echo "Set \$SMARTOS_HOST to the remote zone's IP address"
		exit 1
	fi
}

_main $*
