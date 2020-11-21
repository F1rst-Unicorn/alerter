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

package de.njsm.alerter.test.client;

import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import io.burt.jmespath.Expression;
import io.burt.jmespath.JmesPath;
import io.burt.jmespath.gson.GsonRuntime;
import org.apache.commons.io.IOUtils;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.newsclub.net.unix.AFUNIXServerSocket;
import org.newsclub.net.unix.AFUNIXSocket;

import java.io.IOException;
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.nio.charset.Charset;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.fail;

public class MessageAsserter {

    private static final Logger LOG = LogManager.getLogger(MessageAsserter.class);

    private AFUNIXServerSocket socket;

    private Optional<String> text;

    private Optional<String> title;

    private Optional<String> channel;

    private Optional<String> titleLink;

    private Optional<String> color;

    private Map<String, String> fields;

    private Thread thread;

    private final BlockingQueue<String> queue;
    public MessageAsserter() {
        text = Optional.empty();
        title = Optional.empty();
        channel = Optional.empty();
        titleLink = Optional.empty();
        color = Optional.empty();
        fields = new TreeMap<>();

        queue = new ArrayBlockingQueue<>(1);
    }

    public MessageAsserter hasText(String text) {
        this.text = Optional.of(text);
        return this;
    }

    public MessageAsserter hasTitle(String title) {
        this.title = Optional.of(title);
        return this;
    }

    public MessageAsserter hasChannel(String channel) {
        this.channel = Optional.of(channel);
        return this;
    }

    public MessageAsserter hasTitleLink(String titleLink) {
        this.titleLink = Optional.of(titleLink);
        return this;
    }

    public MessageAsserter hasColor(String color) {
        this.color = Optional.of(color);
        return this;
    }

    public MessageAsserter hasField(String key, String value) {
        fields.put(key, value);
        return this;
    }

    public static MessageAsserter build(AFUNIXServerSocket socket) {
        MessageAsserter result = new MessageAsserter();
        result.socket = socket;
        return result;
    }

    public MessageAsserter start() {
        thread = new Thread(() -> {
            try {
                AFUNIXSocket fdsa = socket.accept();
                String result = IOUtils.toString(fdsa.getInputStream(), Charset.defaultCharset());
                queue.add(result);
            } catch (IOException e) {
                e.printStackTrace();
            }
        });
        thread.start();
        return this;
    }

    public void check() throws InterruptedException {
        thread.join();
        String rawMessage = queue.take();
        LOG.debug("JSON: " + rawMessage);

        JmesPath<JsonElement> path = new GsonRuntime();
        JsonElement message = new GsonBuilder().create().fromJson(rawMessage, JsonObject.class);

        checkText(path, message);
        checkTitle(path, message);
        checkTitleLink(path, message);
        checkChannel(path, message);
        checkColor(path, message);
        checkHostname(path, message);
        checkFields(path, message);
    }

    private void checkText(JmesPath<JsonElement> path, JsonElement message) {
        Expression<JsonElement> matcher = path.compile("attachments[0].text");
        JsonElement actualText = matcher.search(message);
        text.ifPresent(v -> assertEquals(v, actualText.getAsString()));
    }

    private void checkTitle(JmesPath<JsonElement> path, JsonElement message) {
        Expression<JsonElement> matcher = path.compile("attachments[0].title");
        JsonElement actualText = matcher.search(message);
        title.ifPresent(v -> assertEquals(v, actualText.getAsString()));
    }

    private void checkChannel(JmesPath<JsonElement> path, JsonElement message) {
        Expression<JsonElement> matcher = path.compile("channel");
        JsonElement actualText = matcher.search(message);
        channel.ifPresent(v -> assertEquals(v, actualText.getAsString()));
    }

    private void checkTitleLink(JmesPath<JsonElement> path, JsonElement message) {
        Expression<JsonElement> matcher = path.compile("attachments[0].title_link");
        JsonElement actualText = matcher.search(message);
        titleLink.ifPresent(v -> assertEquals(v, actualText.getAsString()));
    }

    private void checkColor(JmesPath<JsonElement> path, JsonElement message) {
        Expression<JsonElement> matcher = path.compile("attachments[0].color");
        JsonElement actualText = matcher.search(message);
        color.ifPresent(v -> assertEquals(v, actualText.getAsString()));
    }

    private void checkHostname(JmesPath<JsonElement> path, JsonElement message) {
        Expression<JsonElement> matcher = path.compile("username");
        JsonElement actualText = matcher.search(message);
        try {
            assertEquals(InetAddress.getLocalHost().getCanonicalHostName(), actualText.getAsString());
        } catch (UnknownHostException e) {
            fail();
        }
    }

    private void checkFields(JmesPath<JsonElement> path, JsonElement message) {
        fields.forEach((k,v) -> {
            Expression<JsonElement> matcher = path.compile("attachments[0].fields[?title == '" + k + "'].value");
            JsonElement actualText = matcher.search(message);
            assertEquals(v, actualText.getAsString());
        });
    }
}
