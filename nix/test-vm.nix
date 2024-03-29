# Demo VM image to test whether this package (and other Azure virtualisation
# features) works. Deploy as a Gen1 VM.

{ config, lib, modulesPath, pkgs, ... }:

{
  imports = [ "${modulesPath}/virtualisation/azure-common.nix" ];

  # Users
  users.mutableUsers = false;
  users.users.admin = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = [ "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILfl45fta8cW2n2+sreVOwZFXsJ3zNp/h47joOn5ctUr" ];
  };
  security.sudo.wheelNeedsPassword = false;
  nix.settings.trusted-users = [ "admin" ];

  # Networking
  networking = {
    hostName = "tiny-azagent-test";
    useDHCP = false;
  };

  systemd.network.enable = true;
  systemd.network.networks."30-eth0" = {
    name = "eth0";
    DHCP = "yes";
  };
  services.openssh.enable = true;

  # Environment
  time.timeZone = "UTC";
  system.stateVersion = "22.11";

  # Azure
  virtualisation.azure.tiny-azagent.enable = true;
  virtualisation.azure.agent.enable = lib.mkForce false;

  systemd.services.consume-hypervisor-entropy = {
    # Based on: https://github.com/NixOS/nixpkgs/blob/c11e244d6e4f84f45b7f86b1396757063b4d8ab5/nixos/modules/virtualisation/azure-agent.nix#L220
    description = "Consume entropy in ACPI table provided by Hyper-V";

    wantedBy = [ "sysinit.target" ];
    before = [ "sysinit.target" ];
    unitConfig.DefaultDependencies = false;
    restartIfChanged = false;

    path = [ pkgs.coreutils ];
    script = ''
      echo "Fetching entropy..."
      cat /sys/firmware/acpi/tables/OEM0 > /dev/random
    '';

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      StandardError = "journal+console";
      StandardOutput = "journal+console";
    };
  };

  services.udev.extraRules = ''
    ENV{DEVTYPE}=="disk", ATTRS{device_id}=="?00000000-0001-*", SYMLINK+="disk/azure-resource"
  '';

  fileSystems."/mnt/resource" = {
    device = "/dev/disk/azure-resource";
    fsType = "ext4";
    autoFormat = true;
  };

  swapDevices = [{ device = "/mnt/resource/swapfile"; size = 1024; }];

  system.build.azureImage = import "${modulesPath}/../lib/make-disk-image.nix" {
    name = "azure-image";
    postVM = ''
      ${pkgs.vmTools.qemu}/bin/qemu-img convert -f raw -o subformat=fixed,force_size -O vpc $diskImage $out/disk.vhd
      rm $diskImage
    '';
    format = "raw";
    diskSize = "auto";
    copyChannel = false;
    inherit config lib pkgs;
  };
}
