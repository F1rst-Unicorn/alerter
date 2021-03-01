/* alerter: Alerter to chat servers
 * Copyright (C) 2019  The alerter developers
 *
 * This file is part of the alerter program suite.
 *
 * alerter is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * alerter is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

package de.njsm.alerter.test.client;

import org.apache.commons.io.IOUtils;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.newsclub.net.unix.AFUNIXServerSocket;
import org.newsclub.net.unix.AFUNIXSocket;

import java.io.IOException;
import java.nio.charset.Charset;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

public class MessageCaptor {

    private static final Logger LOG = LogManager.getLogger(MessageCaptor.class);

    private Thread thread;

    private final BlockingQueue<String> queue;

    private AFUNIXServerSocket socket;

    public MessageCaptor() {
        queue = new ArrayBlockingQueue<>(1);
    }

    public static MessageCaptor build(AFUNIXServerSocket socket) {
        MessageCaptor result = new MessageCaptor();
        result.socket = socket;
        return result;
    }

    public MessageCaptor start() {
        thread = new Thread(() -> {
            try {
                AFUNIXSocket acceptedSocket = socket.accept();
                String result = IOUtils.toString(acceptedSocket.getInputStream(), Charset.defaultCharset());
                queue.add(result);
            } catch (IOException e) {
                LOG.error("Failed to read message", e);
            }
        });
        thread.start();
        return this;
    }

    public MessageAsserter verify() throws InterruptedException {
        thread.join();
        String rawMessage = queue.take();
        LOG.debug("JSON: " + rawMessage);
        return new MessageAsserter(rawMessage);
    }
}
