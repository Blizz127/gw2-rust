// Wine ConnectEx/IOCP probe for the GW2 sandbox (our own test program, not GW2 code).
// usage: connectex-probe.exe <ipv4> <port>
// Mirrors the client's observed sequence: WSASocketW(overlapped) -> FIONBIO ->
// bind(0.0.0.0:0) -> ConnectEx via SIO_GET_EXTENSION_FUNCTION_POINTER -> IOCP wait,
// then SO_UPDATE_CONNECT_CONTEXT and an overlapped WSARecv.
#include <winsock2.h>
#include <mswsock.h>
#include <ws2tcpip.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    if (argc < 3) { printf("usage: %s ip port\n", argv[0]); return 2; }
    WSADATA w; WSAStartup(MAKEWORD(2, 2), &w);
    SOCKET s = WSASocketW(AF_INET, SOCK_STREAM, IPPROTO_TCP, NULL, 0, WSA_FLAG_OVERLAPPED);
    u_long nb = 1; ioctlsocket(s, FIONBIO, &nb);
    struct sockaddr_in any = {0}; any.sin_family = AF_INET;
    if (bind(s, (struct sockaddr *)&any, sizeof any)) { printf("bind failed %d\n", WSAGetLastError()); return 1; }
    GUID g = WSAID_CONNECTEX; LPFN_CONNECTEX cx = NULL; DWORD n = 0;
    WSAIoctl(s, SIO_GET_EXTENSION_FUNCTION_POINTER, &g, sizeof g, &cx, sizeof cx, &n, NULL, NULL);
    HANDLE iocp = CreateIoCompletionPort((HANDLE)s, NULL, 42, 0);
    struct sockaddr_in to = {0}; to.sin_family = AF_INET; to.sin_port = htons((u_short)atoi(argv[2]));
    inet_pton(AF_INET, argv[1], &to.sin_addr);
    OVERLAPPED ov = {0};
    BOOL ok = cx(s, (struct sockaddr *)&to, sizeof to, NULL, 0, NULL, &ov);
    printf("ConnectEx returned %d err %d\n", ok, ok ? 0 : WSAGetLastError());
    DWORD bytes = 0; ULONG_PTR key = 0; LPOVERLAPPED pov = NULL;
    BOOL got = GetQueuedCompletionStatus(iocp, &bytes, &key, &pov, 5000);
    printf("connect completion: got=%d key=%llu err=%lu\n", got, (unsigned long long)key, got ? 0 : GetLastError());
    if (!got) { printf("RESULT: ConnectEx completion NOT delivered within 5s\n"); return 3; }
    setsockopt(s, SOL_SOCKET, SO_UPDATE_CONNECT_CONTEXT, NULL, 0);
    char buf[64]; WSABUF wb = { sizeof buf, buf }; DWORD flags = 0, got_n = 0; OVERLAPPED ov2 = {0};
    int r = WSARecv(s, &wb, 1, &got_n, &flags, &ov2, NULL);
    printf("WSARecv returned %d err %d\n", r, r ? WSAGetLastError() : 0);
    got = GetQueuedCompletionStatus(iocp, &bytes, &key, &pov, 3000);
    printf("recv completion: got=%d bytes=%lu err=%lu\n", got, bytes, got ? 0 : GetLastError());
    printf("RESULT: ConnectEx completion delivered\n");
    closesocket(s);
    return 0;
}
