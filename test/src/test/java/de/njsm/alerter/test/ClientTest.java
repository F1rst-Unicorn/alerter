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

package de.njsm.alerter.test;

import de.njsm.alerter.test.client.Alert;
import de.njsm.alerter.test.client.MessageCaptor;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.newsclub.net.unix.AFUNIXServerSocket;
import org.newsclub.net.unix.AFUNIXSocketAddress;

import java.io.File;
import java.io.IOException;

public class ClientTest {

    private MessageCaptor captor;

    private interface Level {
        String OK = "OK";
        String WARN = "WARN";
        String ERROR = "ERROR";
        String UNKNOWN = "UNKNOWN";
    }

    @BeforeEach
    public void setup() throws IOException {
        File socketFile = new File("alert.sock");
        socketFile.delete();
        AFUNIXServerSocket socket = AFUNIXServerSocket.newInstance();
        socket.bind(new AFUNIXSocketAddress(socketFile), 1);
        captor = MessageCaptor.build(socket).start();
    }

    @Test
    @Timeout(2)
    public void simpleMessageWorks() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .call();

        captor.verify()
                .hasText(text);
    }

    @Test
    @Timeout(2)
    public void messageToExplicitChannelWorks() throws InterruptedException {
        String text = "test";
        String channel = "channel";

        Alert.build()
                .withText(text)
                .withChannel(channel)
                .call();

        captor.verify()
                .hasText(text)
                .hasChannel(channel);
    }

    @Test
    @Timeout(2)
    public void messageWithTitleWorks() throws InterruptedException {
        String text = "test";
        String title = "title";

        Alert.build()
                .withText(text)
                .withTitle(title)
                .call();

        captor.verify()
                .hasText(text)
                .hasTitle(title);
    }

    @Test
    @Timeout(2)
    public void messageWithTitleAndTitleLinkWorks() throws InterruptedException {
        String text = "test";
        String title = "title";
        String titleLink = "titleLink";

        Alert.build()
                .withText(text)
                .withTitle(title)
                .withTitleLink(titleLink)
                .call();

        captor.verify()
                .hasText(text)
                .hasTitle(title)
                .hasTitleLink(titleLink);
    }

    @Test
    @Timeout(2)
    public void noLevelGivesUnknownColor() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .call();

        captor.verify()
                .hasText(text)
                .hasColor(Level.UNKNOWN);
    }

    @Test
    @Timeout(2)
    public void okLevelGivesOkColor() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .withLevel(Level.OK)
                .call();

        captor.verify()
                .hasText(text)
                .hasColor(Level.OK);
    }

    @Test
    @Timeout(2)
    public void warnLevelGivesWarnColor() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .withLevel(Level.WARN)
                .call();

        captor.verify()
                .hasText(text)
                .hasColor(Level.WARN);
    }

    @Test
    @Timeout(2)
    public void errorLevelGivesErrorColor() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .withLevel(Level.ERROR)
                .call();

        captor.verify()
                .hasText(text)
                .hasColor(Level.ERROR);
    }

    @Test
    @Timeout(2)
    public void unknownLevelGivesUnknownColor() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .withLevel(Level.UNKNOWN)
                .call();

        captor.verify()
                .hasText(text)
                .hasColor(Level.UNKNOWN);
    }

    @Test
    @Timeout(2)
    public void keyValuePairsWork() throws InterruptedException {
        String text = "test";

        Alert.build()
                .withText(text)
                .withLevel(Level.ERROR)
                .withField("simple", "simple")
                .withField("key", ":value:test:")
                .call();

        captor.verify()
                .hasText(text)
                .hasField("simple", "simple")
                .hasField("key", ":value:test:");
    }
}
