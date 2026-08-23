#!/bin/bash

XDG_DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}

if [ -d "/opt/system/Tools/PortMaster/" ]; then
  controlfolder="/opt/system/Tools/PortMaster"
elif [ -d "/opt/tools/PortMaster/" ]; then
  controlfolder="/opt/tools/PortMaster"
elif [ -d "$XDG_DATA_HOME/PortMaster/" ]; then
  controlfolder="$XDG_DATA_HOME/PortMaster"
else
  controlfolder="/roms/ports/PortMaster"
fi

source $controlfolder/control.txt
source $controlfolder/device_info.txt
[ -f "${controlfolder}/mod_${CFW_NAME}.txt" ] && source "${controlfolder}/mod_${CFW_NAME}.txt"
get_controls

GAMEDIR=/$directory/ports/dr-rustario-vs-rustris

> "$GAMEDIR/log.txt" && exec > >(tee "$GAMEDIR/log.txt") 2>&1

# config.yml and the high score tables are written next to the binary
cd $GAMEDIR

# the game reads the pad natively; PortMaster supplies the device's SDL controller mapping
export SDL_GAMECONTROLLERCONFIG="$sdl_controllerconfig"

pm_platform_helper "$GAMEDIR/dr-rustario-vs-rustris.${DEVICE_ARCH}"
./dr-rustario-vs-rustris.${DEVICE_ARCH}

pm_finish
