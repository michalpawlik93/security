You can inject any DLL you like into a running process on your PC. The injected DLL must have the DllMain function as its entry point. DllMain is automatically executed once the DLL is loaded by LoadLibraryA from Kernel32.dll.

You can find Kernel32.dll in YourDrive:\Windows\System32. It is attached to all processes running on your PC with Windows. The path to the injected DLL is stored in a raw memory address, making it possible to read it from another process.

If your DLL doesn't work in the remote process, you can download the Process Monitor tool from the Microsoft site: https://learn.microsoft.com/en-us/sysinternals/downloads/procmon to diagnose the issue.
To diagnoze issue. 
1. Your target process must be running.
2. Open ProcessMonitor.
3. Open Tools in toolbar. Next click process Tree...
4. Select your running process and click Include process
![image](https://github.com/user-attachments/assets/cf4b1fa1-9348-4945-8de5-9f52fd7b03c1)

5. You can track now proccess operations.

App contain few programs. You can inject dll to target process or you can load dll to current process. Second option is nice for Injected Dll testing.
