#include <windows.h>
#include <stdio.h>
#include <aclapi.h>

int main() {
    TCHAR filePath[] = TEXT("C:\\path\\to\\your\\file.txt"); // Replace with your file path

    SECURITY_DESCRIPTOR* pSD = NULL;
    DWORD dwLengthNeeded;

    // Get the security descriptor
    if (!GetFileSecurity(filePath, OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION, pSD, 0, &dwLengthNeeded)) {
        if (GetLastError() == ERROR_INSUFFICIENT_BUFFER) {
            pSD = (SECURITY_DESCRIPTOR*)malloc(dwLengthNeeded);
            if (!GetFileSecurity(filePath, OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION, pSD, dwLengthNeeded, &dwLengthNeeded)) {
                printf("Error getting security descriptor: %u\n", GetLastError());
                return 1;
            }
        } else {
            printf("Error getting security descriptor: %u\n", GetLastError());
            return 1;
        }
    }

    // Access the security descriptor information
    // ... (see documentation for details)

    free(pSD);
    return 0;
}
