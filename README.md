# tiny-azagent

**Use at your own risk!**

Minimal provisioning agent for Azure VMs. This thing *only* reports "ready" to Azure. It does not perform any provisioning tasks. This is useful if you are already deploying a fully provisioned image to Azure.

## Installation

*After* network is ready (i.e. eth0 has a DHCP lease), start this program. It will tell Azure that the VM is ready and immediately exit.

Only run one of waagent, cloud-init, or tiny-azagent. Running multiple provisioning agents causes explosions.

A sample systemd unit is kind of in `nix/nixos-module.nix`.

## Disclaimer

This is unsupported by Microsoft. You should probably use [waagent](https://github.com/azure/WALinuxAgent) or [cloud-init](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/using-cloud-init) instead.

## What's done

Using the server at http://168.63.129.16:

1. Fetches the "goal state" (`GET /machine?comp=goalstate`)
2. Reports ready (`POST /machine?comp=health`)

As explained in the [Azure VM docs](https://learn.microsoft.com/en-us/azure/virtual-machines/linux/no-agen).

## What's not done

Since this agent does so little, to have a functioning VM you must perform *at least* these tasks:

1. **You must manually consume hypervisor entropy** (e.g. `cat /sys/firmware/acpi/tables/OEM0 > /dev/random`). Not doing this results in an extremely insecure system.
2. You must manually configure networking, and ideally publish the system hostname via DHCP Requests (networkd does this).
3. Your disk image must have preconfigured users. This agent does not install SSH keys or root passwords.
4. If you want to use the temporary resource disk, you must manually mount/format it (e.g. udev disk with [`ATTRS{device_id}=="?00000000-0001-*"`](https://github.com/Azure/WALinuxAgent/blob/04ded9f0b708cfaf4f9b68eead1aef4cc4f32eeb/config/66-azure-storage.rules#L9)

A VM configuration demonstrating this is in `nix/test-vm.nix`.

## License

[MIT](COPYING)
