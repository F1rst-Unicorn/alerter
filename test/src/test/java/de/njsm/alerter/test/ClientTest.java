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

    private interface ColorCode {
        String OK = "#44bb77";
        String WARN = "#ffaa44";
        String ERROR = "#ff5566";
        String UNKNOWN = "#aa44ff";
    }

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
                .hasText(text)
                .hasHostname();
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
                .hasChannel(channel)
                .hasHostname();
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
                .hasTitle(title)
                .hasHostname();
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
                .hasTitleLink(titleLink)
                .hasHostname();
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
                .hasColor(ColorCode.UNKNOWN)
                .hasHostname();
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
                .hasColor(ColorCode.OK)
                .hasHostname();
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
                .hasColor(ColorCode.WARN)
                .hasHostname();
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
                .hasColor(ColorCode.ERROR)
                .hasHostname();
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
                .hasColor(ColorCode.UNKNOWN)
                .hasHostname();
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
                .hasField("key", ":value:test:")
                .hasHostname();
    }
}
