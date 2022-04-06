/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

// This file:
// Created: 2022-04-04T08:37:16+02:00 by Hendrik G. Seliger (github@hseliger.eu)
// Last changes: 2022-04-05T11:46:39+02:00 by Hendrik G. Seliger (github@hseliger.eu)

// Based on an example given by Ranjit Katuri on https://stackoverflow.com/a/17645247
// Uses RapidJSON header-only parser (http://rapidjson.org/)

// Error handling is quite rudimentary: whenever something goes wrong, the functions simply
// return a somewhat generic message instead of the active window's title, class, or command.

#define DEBUG 0

#include "native.h"
#include <unistd.h> // for readlink
#include <iostream>
#include <dbus/dbus.h>
#include <assert.h>
#include <cstring>

DBusConnection* conn = NULL;

#define DB_INTERFACE    "org.gnome.Shell.Extensions.WindowsExt"
#define DB_DESTINATION  "org.gnome.Shell"
#define DB_PATH         "/org/gnome/Shell/Extensions/WindowsExt"

// Careful!! Code relies on methods aligning with INFO-numbers below!!
#define INFO_TITLE      1
#define INFO_EXEC       2
#define INFO_WINCLASS   3
static const char* methods[] = { "FocusTitle", "FocusPID", "FocusClass" };

#define MAX_CMD_LINE    120

static const char* errMessage = "Error retrieving window info. Are you on Gnome and do you have the Window Calls extension installed and active?";

// Helper function to setup connection
int _vsetupconnection()
{
    DBusError err;
    // initialise the errors
    dbus_error_init(&err);
    // connect to session bus
    conn = dbus_bus_get(DBUS_BUS_SESSION, &err);
    if (dbus_error_is_set(&err)) {
        if (DEBUG > 0) {
        ::perror("Connection Error. ");
        ::perror(err.name);
        ::perror(err.message);
        }
        dbus_error_free(&err);
    }
    if (NULL == conn) {
        return(-1);
    }
    else   {
        if (DEBUG > 0) {
            std::cerr << "Connected to session bus\n";
        }
        return(1);
    }
}

//Send method call, Returns NULL on failure, else pointer to reply
// DBusMessage* _sendMethodCall(const char* objectpath,
//         const char* busname,
//         const char* interfacename,
//         const char* methodname);
DBusMessage* _sendMethodCall(const char* objectpath, const char* busname, const char* interfacename, const char* methodname)
{
    assert(objectpath != NULL); assert(busname != NULL);    assert(interfacename != NULL);
    assert(methodname != NULL); assert(conn != NULL);

    DBusMessage* methodcall = dbus_message_new_method_call(busname,objectpath, interfacename, methodname);

    if (methodcall == NULL)    {
        if (DEBUG > 0) {
        ::perror("Cannot allocate DBus message!");
        }
        return(NULL);
    }
    //Now do a sync call
    DBusPendingCall* pending;
    DBusMessage* reply;

    //Send and expect reply using pending call object
    if (!dbus_connection_send_with_reply(conn, methodcall, &pending, -1))
    {
        if (DEBUG == 1) {
        ::perror("failed to send message!");
        }
        return(NULL);
    }
    dbus_connection_flush(conn);
    dbus_message_unref(methodcall);
    methodcall = NULL;

    //Now block on the pending call
    dbus_pending_call_block(pending);
    //Get the reply message from the queue
    reply = dbus_pending_call_steal_reply(pending);
    //Free pending call handle
    dbus_pending_call_unref(pending);
    assert(reply != NULL);

    if(dbus_message_get_type(reply) ==  DBUS_MESSAGE_TYPE_ERROR)    {
        if (DEBUG >0 ) {
        ::perror("Error!");
        ::perror(dbus_message_get_error_name(reply));
        }
        dbus_message_unref(reply);
        reply = NULL;
    }
    return reply;
}

void _getInformation(int infoType, char *buffer, int32_t buffer_size) {
    // First, ensure we get a connection. Then, which should never happen,
    // but just in case ensure we have a method for the infoType
    if ( (_vsetupconnection() == 1) && (infoType <= (int)sizeof(methods)) ) {
        std::cerr << "Using method " << methods[infoType - 1] << "\n";
        DBusMessage* reply = _sendMethodCall(DB_PATH, DB_DESTINATION, DB_INTERFACE, methods[infoType - 1]);
        if(reply != NULL)    {
            DBusMessageIter MsgIter;
            dbus_message_iter_init(reply, &MsgIter);//msg is pointer to dbus message received

            if (DBUS_TYPE_STRING == dbus_message_iter_get_arg_type(&MsgIter)){
                char* dbusMsg = NULL;
                dbus_message_iter_get_basic(&MsgIter, &dbusMsg);

                if (DEBUG > 1) {
                    std::cerr << "Received string: " << dbusMsg << "\n";
                }
                switch (infoType) {
                    case INFO_TITLE:
                        // std::cout << "Title: " << (*p)["title"].GetString() << "\n";
                        strncpy(buffer, dbusMsg, buffer_size);
                        break;
                    case INFO_EXEC:
                        {
                            size_t pathLen  = snprintf(NULL, 0, "/proc/%s/cmdline", dbusMsg);
                            char* pathBuffer = (char*) malloc(pathLen);
                            sprintf(pathBuffer,"/proc/%s/exe", dbusMsg);
                            // std::cout << "Proc file: " << pathBuffer << "\n";
                            readlink(pathBuffer, buffer, buffer_size);
                            free(pathBuffer);
                        }
                        break;
                    case INFO_WINCLASS:
                        // std::cout << "Class: " << (*p)["class"].GetString() << "\n";
                        strncpy(buffer, dbusMsg, buffer_size);
                        break;
                    default:
                        strncpy(buffer, errMessage, buffer_size);
                }
                return;
            }
            dbus_message_unref(reply); //unref reply
        } else {
            if (DEBUG == 1) {
            ::perror("Error! Send Message Call failed!");
            }
            strncpy(buffer, errMessage, buffer_size);
        }
        // Closing gives error: Applications must not close shared connections - see dbus_connection_close() docs.
        // dbus_connection_close(conn);

    } else {
        if (DEBUG > 0) {
        ::perror("Error! Could not get connection to session bus!");
        }
        strncpy(buffer, errMessage, buffer_size);
    }
}


int32_t info_get_title(char *buffer, int32_t buffer_size)
{
    _getInformation(INFO_TITLE, buffer, buffer_size);
    return 1;
}

int32_t info_get_exec(char *buffer, int32_t buffer_size)
{
    _getInformation(INFO_EXEC, buffer, buffer_size);
    return 1;
}

int32_t info_get_class(char *buffer, int32_t buffer_size)
{
    _getInformation(INFO_WINCLASS, buffer, buffer_size);
    return 1;
}

#if DEBUG > 1
int main (int argc, char **argv)
{
    (void)argc;
    (void)argv;

    char outString[MAX_CMD_LINE];

    info_get_title(outString, MAX_CMD_LINE);
    std::cout << outString << "\n";
    info_get_exec(outString, MAX_CMD_LINE);
    std::cout << outString << "\n";
    info_get_class(outString, MAX_CMD_LINE);
    std::cout << outString << "\n";

    return 0;
}
#endif