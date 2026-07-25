---
title: "PREEMPT_RT is finally in Linux!"
date: "2026-05-18"
excerpt: "A new era for critical embedded applications begins"
tags: ["RealTime","linux","Operating System"]
read_time: 10
image: "https://miro.medium.com/v2/resize:fit:4800/format:webp/1*KnIX1_PMhz-fHUwJpV9M0A.png"
---

While ago I wrote about Zephyr an RTOS ( currently a project under de umbrella of Linux Foundation). It has been quite a bit, but following the latest news, last week was finally merged quite interesting features into the Linux kernel (Linux 6.12-rc1) like QR code Panic messages, sched_ext and the most awaited feature PREEMPT_RT (Real Time Linux).


Although for a couple of years, Archlinux has been shipping a patched kernel with this feature, now it’s oficially mainline.

Since Linux 5.15, PREEMPT_RT has been partially merged into the mainline Linux kernel.

but… What exactly is an RTOS?

It’s an OS that focused explicitly to process real time applications.

Its a time-bound system which has well-defined, fixed time contraints, where the processing must be done within the defined constraints or the system will fail.

gotcha!, but now … What is preemption?

In simple words, is the ability to stop whatever the CPU is running to run another task, and in an RTOS any task should be preemptible, both in userspace and kernelspace.

Now that the merge windows are closed , the focus shifts to testing. I encourage you to download and try the Release Candidate kernel from Linus’ git tree.

![Index](https://miro.medium.com/v2/resize:fit:720/format:webp/1*3hAthiZG7GkYsO9-awtfow.png)

Well, it’s time to compile it and test it!

First, download and extract the kernel source.

I always remember this comics, is still kind of funny.

![meme](https://miro.medium.com/v2/resize:fit:720/format:webp/0*iIgVISUKvf-bViq1.png)

```bash
$ tar -xzf Linux-6.12-rc1.tar.gz
```

Lets add the features that we actually want to try.

```bash
$ make menuconfig
```

Currently my kernel was compiled with “Voluntary Kernel Preemption (Desktop)”, so if there are a specific process that yields “voluntarily” the CPU it will call yield() system call, then the kernel checks if there are other processes that are ready to run. If there are other processes, the kernel will perform a context switch and load the state of the process.

![menu](https://miro.medium.com/v2/resize:fit:720/format:webp/1*QDkj4567EUpuM_9RyOpwdQ.png)

In this case high priority tasks will always preempt a low priority tasks, at userspace and kernelspace, ensuring that critical operations are executed promptly.

![menu1](https://miro.medium.com/v2/resize:fit:720/format:webp/1*eEE7GCOkM4d6ELTR77yhoQ.png)

Once you selected all the values, it’s compilation time!

```bash
make
sudo make modules_install
sudo make install
```

This will generate this files, initramfs, system.map and vmlinuz.

Now, its time to modify the grub loader.

```bash
sudo grub2-mkconfig -o /boot/grub2/grub.cfg
```

Don’t forget to validate that you create a valid entry.
```bash
grubby --info=ALL 
```

and reboot it!

Siehe da! , now testing time!

![uname](https://miro.medium.com/v2/resize:fit:720/format:webp/1*JeYnUcqcpGEg6F2PpBPLvQ.png)

In order to test our new build kernel, we’ll need some components that may be available in your favorite distribution, well that wasn’t my case, so you also can find those utilities here. (RT-TESTS)

[Linux @ CERN](https://linuxsoft.cern.ch/cern/centos/7/rt/x86_64/repoview/rt-tests.html?source=post_page-----27c0c789a4e5---------------------------------------)

With cyclictest we can test the latency, where the most important value is the maximum detected latency, because this can give an idea of the worst case latency, in my case 2016 us, that is a quite high value.

![cyclic](https://miro.medium.com/v2/resize:fit:720/format:webp/1*8vK6BXfu18G4jn3GxCMECQ.png)

But now, let’s make it real, the goal is to ilustrate how a high priority task will preempt a low priority task.

The premise is simple, lets run two tasks concurrently, where one last 0.5 and the other 1.0, and assign a priorities to each one of them.

```bash
#include <stdio.h>
#include <pthread.h>
#include <sched.h>
#include <time.h>
#include <unistd.h>
// This task will take 0.5 seconds 
void* high_priority_task(void* arg) {
    while (1) {
        printf("High-priority task running\n");
        usleep(500000);
    }
}
//Meanwhile this task will take 1 second.
void* low_priority_task(void* arg) {
    while (1) {
        printf("Low-priority task running\n");
        usleep(1000000);
    }
}

int main() {
    pthread_t thread1, thread2;
    struct sched_param param1, param2;
    pthread_attr_t attr1, attr2;
    pthread_attr_init(&attr1);
    pthread_attr_init(&attr2);

    // Here happens the magic!!!
    pthread_attr_setschedpolicy(&attr1, SCHED_FIFO);
    pthread_attr_setschedpolicy(&attr2, SCHED_FIFO);

    // Set major priority to task1 over task2
    param1.sched_priority = 80;
    param2.sched_priority = 60;
    pthread_attr_setschedparam(&attr1, &param1);
    pthread_attr_setschedparam(&attr2, &param2);

    // To simulate concurrent tasks, we create threads
    pthread_create(&thread1, &attr1, high_priority_task, NULL);
    pthread_create(&thread2, &attr2, low_priority_task, NULL);

   pthread_join(thread1, NULL);
    pthread_join(thread2, NULL);
}
```

So the finals results, as we can see the high-priority task will preempt over the Low-priority task.

![results](https://miro.medium.com/v2/resize:fit:640/format:webp/1*uloV01Wtjw507FrhJ9YIlA.png)

This is huge!, this opens up new possibilities for Linux kernel in a wider range of applications that require this kind of deterministic performance. Since IoT devices, Automotive Grade Linux, Healthcare, and obviously embedded military applications (missile control).

