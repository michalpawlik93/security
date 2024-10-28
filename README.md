Repository:

I. dll_inject
II. shell_code_inject
III. web_server_vulnerabilities

I.
dll_inject : This application is designed for injecting DLL files into running processes. Upon launching the program, four modes will be presented. Two of these modes load a DLL into the current process, allowing for faster DLL testing. By loading the DLL into the current process, you can diagnose issues more efficiently without needing to log data to an external file. The other two modes are used for injecting a DLL into a different process. The first library, injected_lib.dll, creates a file at a specific path that must be changed in the program before compilation. The second library, keylogger_lib.dll, listens for keyboard inputs and records the information to a file.

    Program description: key_logger_lib.dll injection
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

II.
shell_code_inject : This application is designed to inject shellcode written in assembly into a running process.

III.
web_server_vulnerabilities : This application creates a web server that is subjected to typical web attacks. The project consists of separate folders, each simulating a specific attack on the web server. The web server is designed to implement security features that will be activated using feature flags.
The web server uses the mio crate, which provides solutions integrated with IOCP (I/O Completion Ports). This ensures asynchronous handling of sockets.
