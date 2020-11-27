/* alerter: Alerter to Slack
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

package de.njsm.alerter.test;

import de.njsm.alerter.test.client.Alert;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockserver.client.MockServerClient;
import org.mockserver.junit.jupiter.MockServerExtension;
import org.mockserver.junit.jupiter.MockServerSettings;
import org.mockserver.matchers.Times;
import org.mockserver.model.HttpResponse;
import org.mockserver.model.MediaType;
import org.mockserver.verify.VerificationTimes;
import org.newsclub.net.unix.AFUNIXSocket;
import org.newsclub.net.unix.AFUNIXSocketAddress;

import java.io.*;
import java.nio.charset.Charset;
import java.util.concurrent.atomic.AtomicBoolean;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockserver.model.HttpRequest.request;

@ExtendWith(MockServerExtension.class)
@MockServerSettings(ports = {17553})
public class ServerTest {

    private static final String QUEUE_PATH = "src/test/resources/queue";

    private Process alerter;

    private Thread logDispatcherOut;

    private Thread logDispatcherErr;

    private AtomicBoolean keepRunning;

    @BeforeEach
    public void setup(MockServerClient client) throws IOException {
        client.reset();
        new File(QUEUE_PATH).delete();

        alerter = Runtime.getRuntime().exec(new String[]{
                Config.getServerBinaryPath(),
                "-c",
                Config.getConfigPath(),
                "-v",
                Config.getLogConfigPath()
        });

        keepRunning = new AtomicBoolean(true);

        logDispatcherOut = new Thread(() -> {
            while (keepRunning.get()) {
                try {
                    alerter.getInputStream().transferTo(System.out);
                } catch (IOException e) {
                    e.printStackTrace();
                }
            }
        });
        logDispatcherOut.start();

        logDispatcherErr = new Thread(() -> {
            while (keepRunning.get()) {
                try {
                    alerter.getErrorStream().transferTo(System.err);
                } catch (IOException e) {
                    e.printStackTrace();
                }
            }
        });
        logDispatcherErr.start();
    }

    @AfterEach
    void tearDown() {
        keepRunning.set(false);
        logDispatcherOut.interrupt();
        logDispatcherErr.interrupt();
        alerter.destroy();
    }

    @Test
    public void simpleMessageWorks(MockServerClient client) throws Exception {
        prepareOkReply(client);

        Alert.build()
                .withText("simple")
                .call();

        Thread.sleep(1000);
        client.verify(
                request().withPath("/slack")
                        .withContentType(MediaType.APPLICATION_JSON)
        );
    }

    @Test
    public void failedTransmissionIsRetried(MockServerClient client) throws Exception {
        prepareServerErrorReply(client);
        prepareServerErrorReply(client);
        prepareOkReply(client);

        Alert.build()
                .withText("simple")
                .call();

        Thread.sleep(10000);

        client.verify(
                request().withPath("/slack")
                        .withContentType(MediaType.APPLICATION_JSON),
                VerificationTimes.exactly(3)
        );

        assertEmptyQueue();
    }

    @Test
    void illegalMessageIsIgnored(MockServerClient client) throws Exception {
        File socketFile = new File(Config.SOCKET_PATH);
        AFUNIXSocket sock = AFUNIXSocket.newInstance();
        sock.connect(new AFUNIXSocketAddress(socketFile));

        IOUtils.write("illegal message", sock.getOutputStream(), Charset.defaultCharset());
        Thread.sleep(1000);

        client.verifyZeroInteractions();
        assertEmptyQueue();
    }

    private void assertEmptyQueue() throws IOException {
        try {
            assertEquals("", IOUtils.toString(new FileInputStream(QUEUE_PATH), Charset.defaultCharset()));
        } catch (FileNotFoundException e) {}
    }

    private void prepareOkReply(MockServerClient client) {
        prepareReply(client, 200);
    }

    private void prepareServerErrorReply(MockServerClient client) {
        prepareReply(client, 500);
    }

    private void prepareReply(MockServerClient client, int i) {
        client.when(request()
                        .withMethod("POST"),
                Times.once()
        ).respond(
                HttpResponse.response()
                        .withStatusCode(i)
        );
    }
}
