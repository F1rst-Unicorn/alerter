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

import java.net.InetAddress;
import java.net.UnknownHostException;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.fail;

public class MessageAsserter {

    private final JmesPath<JsonElement> path;

    private final JsonElement message;

    public MessageAsserter(String rawMessage) {
        path = new GsonRuntime();
        message = new GsonBuilder().create().fromJson(rawMessage, JsonObject.class);
    }

    public MessageAsserter hasText(String text) {
        Expression<JsonElement> matcher = path.compile("attachments[0].text");
        JsonElement actualText = matcher.search(message);
        assertEquals(text, actualText.getAsString());
        return this;
    }

    public MessageAsserter hasTitle(String title) {
        Expression<JsonElement> matcher = path.compile("attachments[0].title");
        JsonElement actualText = matcher.search(message);
        assertEquals(title, actualText.getAsString());
        return this;
    }

    public MessageAsserter hasChannel(String channel) {
        Expression<JsonElement> matcher = path.compile("channel");
        JsonElement actualText = matcher.search(message);
        assertEquals(channel, actualText.getAsString());
        return this;
    }

    public MessageAsserter hasTitleLink(String titleLink) {
        Expression<JsonElement> matcher = path.compile("attachments[0].title_link");
        JsonElement actualText = matcher.search(message);
        assertEquals(titleLink, actualText.getAsString());
        return this;
    }

    public MessageAsserter hasColor(String color) {
        Expression<JsonElement> matcher = path.compile("attachments[0].color");
        JsonElement actualText = matcher.search(message);
        assertEquals(color, actualText.getAsString());
        return this;
    }

    public MessageAsserter hasField(String key, String value) {
        Expression<JsonElement> matcher = path.compile("attachments[0].fields[?title == '" + key + "'].value");
        JsonElement actualText = matcher.search(message);
        assertEquals(value, actualText.getAsString());
        return this;
    }

    public void hasHostname() {
        Expression<JsonElement> matcher = path.compile("username");
        JsonElement actualText = matcher.search(message);
        try {
            assertEquals(InetAddress.getLocalHost().getCanonicalHostName(), actualText.getAsString());
        } catch (UnknownHostException e) {
            fail();
        }
    }
}
