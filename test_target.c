#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>

int main() {
    // Values to search for in memory - all data types
    int secret_number = 12345;
    int health_points = 100;
    int score = 999;
    long long big_number = 9876543210LL;
    long long coins = 1000000LL;
    float player_x = 42.5f;
    float speed = 15.75f;
    double balance = 1337.1337;
    double experience = 9999.9999;
    char username[32] = "testplayer";
    char weapon[16] = "sword";
    char location[20] = "dungeon";
    
    // Print process info
    printf("=== Memscan Enhanced Test Target ===\n");
    printf("PID: %d\n", getpid());
    printf("This process contains the following values in memory:\n\n");
    
    printf("32-bit Integers (i32):\n");
    printf("  secret_number = %d\n", secret_number);
    printf("  health_points = %d\n", health_points);
    printf("  score = %d\n", score);
    
    printf("\n64-bit Integers (i64):\n");
    printf("  big_number = %lld\n", big_number);
    printf("  coins = %lld\n", coins);
    
    printf("\n32-bit Floats (f32):\n");
    printf("  player_x = %.1f\n", player_x);
    printf("  speed = %.2f\n", speed);
    
    printf("\n64-bit Floats (f64):\n");
    printf("  balance = %.4f\n", balance);
    printf("  experience = %.4f\n", experience);
    
    printf("\nStrings:\n");
    printf("  username = '%s'\n", username);
    printf("  weapon = '%s'\n", weapon);
    printf("  location = '%s'\n", location);
    
    printf("\nMemory addresses (for reference):\n");
    printf("  secret_number at: %p\n", (void*)&secret_number);
    printf("  health_points at: %p\n", (void*)&health_points);
    printf("  score at: %p\n", (void*)&score);
    printf("  big_number at: %p\n", (void*)&big_number);
    printf("  coins at: %p\n", (void*)&coins);
    printf("  player_x at: %p\n", (void*)&player_x);
    printf("  speed at: %p\n", (void*)&speed);
    printf("  balance at: %p\n", (void*)&balance);
    printf("  experience at: %p\n", (void*)&experience);
    printf("  username at: %p\n", (void*)&username);
    printf("  weapon at: %p\n", (void*)&weapon);
    printf("  location at: %p\n", (void*)&location);
    printf("\n");
    
    // Keep the program running so we can scan it
    printf("=== Test Instructions ===\n");
    printf("1. Run memscan in another terminal\n");
    printf("2. Attach to this process (PID: %d)\n", getpid());
    printf("3. Try scanning for these values:\n");
    printf("   - i32: 12345, 100, 999\n");
    printf("   - i64: 9876543210, 1000000\n");
    printf("   - f32: 42.5, 15.75\n");
    printf("   - f64: 1337.1337, 9999.9999\n");
    printf("   - String: testplayer, sword, dungeon\n");
    printf("\nPress Enter to change values, or Ctrl+C to exit...\n");
    getchar();
    
    // Change some values
    secret_number = 54321;
    health_points = 75;
    score = 1500;
    big_number = 1111111111LL;
    coins = 2000000LL;
    player_x = 99.9f;
    speed = 25.0f;
    balance = 9999.9999;
    experience = 12345.6789;
    strcpy(username, "newplayer");
    strcpy(weapon, "axe");
    strcpy(location, "castle");
    
    printf("\n=== Values Changed! ===\n");
    printf("i32 values:\n");
    printf("  secret_number = %d (was 12345)\n", secret_number);
    printf("  health_points = %d (was 100)\n", health_points);
    printf("  score = %d (was 999)\n", score);
    
    printf("\ni64 values:\n");
    printf("  big_number = %lld (was 9876543210)\n", big_number);
    printf("  coins = %lld (was 1000000)\n", coins);
    
    printf("\nf32 values:\n");
    printf("  player_x = %.1f (was 42.5)\n", player_x);
    printf("  speed = %.1f (was 15.75)\n", speed);
    
    printf("\nf64 values:\n");
    printf("  balance = %.4f (was 1337.1337)\n", balance);
    printf("  experience = %.4f (was 9999.9999)\n", experience);
    
    printf("\nString values:\n");
    printf("  username = '%s' (was 'testplayer')\n", username);
    printf("  weapon = '%s' (was 'sword')\n", weapon);
    printf("  location = '%s' (was 'dungeon')\n", location);
    
    printf("\nPress Enter to exit...\n");
    getchar();
    
    return 0;
}