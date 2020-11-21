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

import org.junit.Before;
import org.junit.Test;
import org.newsclub.net.unix.AFUNIXServerSocket;
import org.newsclub.net.unix.AFUNIXSocketAddress;

import java.io.File;
import java.io.IOException;

public class ClientTest {

    private AFUNIXServerSocket socket;

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

    @Before
    public void setup() throws IOException {
        File socketFile = new File("alert.sock");
        socketFile.delete();
        socket = AFUNIXServerSocket.newInstance();
        socket.bind(new AFUNIXSocketAddress(socketFile), 1);
    }

    @Test
    public void simpleMessageWorks() throws InterruptedException {
        String text = "test";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .start();

        Alert.build()
                .withText(text)
                .call();

        asserter.check();
    }

    @Test
    public void messageToExplicitChannelWorks() throws InterruptedException {
        String text = "test";
        String channel = "channel";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasChannel(channel)
                .start();

        Alert.build()
                .withText(text)
                .withChannel(channel)
                .call();

        asserter.check();
    }

    @Test
    public void messageWithTitleWorks() throws InterruptedException {
        String text = "test";
        String title = "title";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasTitle(title)
                .start();

        Alert.build()
                .withText(text)
                .withTitle(title)
                .call();

        asserter.check();
    }

    @Test
    public void messageWithTitleAndTitleLinkWorks() throws InterruptedException {
        String text = "test";
        String title = "title";
        String titleLink = "titleLink";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasTitle(title)
                .hasTitleLink(titleLink)
                .start();

        Alert.build()
                .withText(text)
                .withTitle(title)
                .withTitleLink(titleLink)
                .call();

        asserter.check();
    }

    @Test
    public void noLevelGivesUnknownColor() throws InterruptedException {
        String text = "test";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasColor(ColorCode.UNKNOWN)
                .start();

        Alert.build()
                .withText(text)
                .call();

        asserter.check();
    }

    @Test
    public void okLevelGivesOkColor() throws InterruptedException {
        String text = "test";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasColor(ColorCode.OK)
                .start();

        Alert.build()
                .withText(text)
                .withLevel(Level.OK)
                .call();

        asserter.check();
    }

    @Test
    public void warnLevelGivesWarnColor() throws InterruptedException {
        String text = "test";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasColor(ColorCode.WARN)
                .start();

        Alert.build()
                .withText(text)
                .withLevel(Level.WARN)
                .call();

        asserter.check();
    }

    @Test
    public void errorLevelGivesErrorColor() throws InterruptedException {
        String text = "test";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasColor(ColorCode.ERROR)
                .start();

        Alert.build()
                .withText(text)
                .withLevel(Level.ERROR)
                .call();

        asserter.check();
    }

    @Test
    public void keyValuePairsWork() throws InterruptedException {
        String text = "test";

        MessageAsserter asserter = MessageAsserter.build(socket)
                .hasText(text)
                .hasField("simple", "simple")
                .hasField("key", ":value:test:")
                .start();

        Alert.build()
                .withText(text)
                .withLevel(Level.ERROR)
                .withField("simple", "simple")
                .withField("key", ":value:test:")
                .call();

        asserter.check();
    }
}
