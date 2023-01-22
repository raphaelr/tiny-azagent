{ config, lib, pkgs, ... }:

let
  cfg = config.virtualisation.azure.tiny-azagent;
in

{
  # interface
  options = with lib; {
    virtualisation.azure.tiny-azagent = {
      enable = mkEnableOption (mdDoc ''
        Enables tiny-azagent, a minimal Azure provisioning agent whose only
        feature is to report "ready" to the Azure fabric.
      '');
    };
  };

  # implementation
  config = lib.mkIf cfg.enable {
    systemd.services.tiny-azagent = {
      # Requires connectivity to the wireserver (168.63.129.16)
      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = lib.getExe pkgs.tiny-azagent;
        Type = "oneshot";
        RemainAfterExit = true;

        DynamicUser = true;
        RemoveIPC = true;
        CapabilityBoundingSet = "";
        PrivateTmp = true;
        PrivateDevices = true;
        PrivateUsers = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        ProtectClock = true;
        ProtectHostname = true;
        ProtectProc = "invisible";
        ProcSubset = "pid";
        RestrictAddressFamilies = [ "AF_INET" ];
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        RestrictRealtime = true;
        RestrictNamespaces = true;
        SystemCallFilter = [ "@system-service" "~@privileged" ];
        SystemCallArchitectures = "native";
      };
    };
  };
}
