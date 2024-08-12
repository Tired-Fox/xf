#include <windows.h>
#include <stdio.h>
#include <aclapi.h>

int main() {
    TCHAR filePath[] = L"C:\\path\\to\\your\\file.txt"; // Replace with your file path
    DWORD dwRes;
    PSECURITY_DESCRIPTOR pSD = NULL;

    // Get the security descriptor for the file
    dwRes = GetFileSecurity(filePath, DACL_SECURITY_INFORMATION, pSD, 0, &dwLengthNeeded);
    if (dwRes == ERROR_INSUFFICIENT_BUFFER) {
        pSD = (PSECURITY_DESCRIPTOR)LocalAlloc(LPTR, dwLengthNeeded);
        if (pSD == NULL) {
            printf("Error allocating memory\n");
            return 1;
        }

        dwRes = GetFileSecurity(filePath, DACL_SECURITY_INFORMATION, pSD, dwLengthNeeded, &dwLengthNeeded);
    }

    if (dwRes != 0) {
        PACL pDacl = NULL;
        BOOL bDaclPresent, bDaclDefaulted;

        // Get the DACL from the security descriptor
        if (GetSecurityDescriptorDacl(pSD, &bDaclPresent, &pDacl, &bDaclDefaulted)) {
            // Process the DACL to get specific permissions
            // (e.g., use GetAce to get individual ACEs)
            // ...
        } else {
            printf("Error getting DACL\n");
        }

        LocalFree(pSD);
    } else {
        printf("Error getting file security\n");
    }

    return 0;
}
