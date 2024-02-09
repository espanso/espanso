> TODO: this document is relative to version 1 and will be updated soon for changes introduced in version 2
>
> Despite significant architectural differences, the following points are still a good approximation
> of the internals.

# Security

Espanso has always been designed with a strong focus on security. 
In the following section, there is an overview of the critical security
components.

If you have any doubt, don't hesitate to contact me.

## Architecture

In its most basic form, a text expander is composed of two parts:

* A **global key detector** that intercepts the keys pressed by the user, in order to determine if a trigger was typed.

* A **key injection mechanism** that injects the final text into the current application, a process known as *expansion*.

At this point, some of you may think that espanso is acting as a key-logger, due to the *global key detector* we mentioned before. The good news is, **it's not!**

While espanso detects key presses as a keylogger would do, **it doesn't log anything**. Moreover, to further reduce risks, for regular matches, espanso only stores in memory the last five chars by default (you can change this amount by setting the `backspace_limit` parameter in the config), and this is needed to allow the user to correct wrongly typed triggers by pressing backspace, up to 5 characters. For `regex` matches the system can store up to 30 characters.

The matching part is implemented with an efficient [data structure](https://github.com/espanso/espanso/blob/master/src/matcher/scrolling.rs) 
that keeps track of the compatible matches in a "rolling" basis. So that in the worst case scenario,
the longest sequence of chars kept in memory would be equal to the longest trigger.

And of course, if you don't trust me you can examine all the code! That's
the wonderful thing about open source :)

### Implementation

The *global key detector* is implemented on top of various OS-dependent APIs, in particular:

* On Windows, it uses the [RawInput API](https://docs.microsoft.com/en-us/windows/win32/inputdev/raw-input).
* On macOS, it uses [addGlobalMonitorForEvents](https://developer.apple.com/documentation/appkit/nsevent/1535472-addglobalmonitorforevents).
* On Linux, it uses the [X Record Extension](https://www.x.org/releases/X11R7.6/doc/libXtst/recordlib.html).

## Reporting Security Issues

To report a security issue, please email me at federicoterzi96[at]gmail.com