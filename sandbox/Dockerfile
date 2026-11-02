# Offline GW2 client sandbox: Wine + DXVK on Mesa lavapipe (software Vulkan) + Xvfb.
# The image holds no client files; the frozen install is mounted read-only at run time.
# Run only with `--network none` (see run.sh / docs/SANDBOX.md).
FROM debian:trixie-slim

ARG DXVK_VERSION=3.1.1
ENV DEBIAN_FRONTEND=noninteractive \
    WINEPREFIX=/sandbox/prefix \
    WINEARCH=win64 \
    WINEDEBUG=-all \
    DISPLAY=:99 \
    VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json

RUN dpkg --add-architecture i386 \
 && apt-get update \
 && apt-get install -y --no-install-recommends \
      wine wine64 libwine fonts-wine \
      mesa-vulkan-drivers vulkan-tools \
      xvfb xauth x11-utils x11-apps netpbm \
      ca-certificates curl tini procps gdb xdotool libc6:i386 libgcc-s1:i386 libstdc++6:i386 libfreetype6 libfreetype6:i386 \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL -o /tmp/dxvk.tgz \
      "https://github.com/doitsujin/dxvk/releases/download/v${DXVK_VERSION}/dxvk-${DXVK_VERSION}.tar.gz" \
 && mkdir -p /opt && tar -C /opt -xzf /tmp/dxvk.tgz && rm /tmp/dxvk.tgz \
 && ln -s /opt/dxvk-${DXVK_VERSION} /opt/dxvk

# python3 only runs the launcher's generated bubble bridge in sandbox runs (BUBBLE_DIR).
RUN apt-get update && apt-get install -y --no-install-recommends python3-minimal && rm -rf /var/lib/apt/lists/*

COPY entry.sh /usr/local/bin/entry.sh
RUN chmod 755 /usr/local/bin/entry.sh && mkdir -p /sandbox && chmod 1777 /sandbox
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/entry.sh"]
CMD ["preflight"]
