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

import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import io.burt.jmespath.Expression;
import io.burt.jmespath.JmesPath;
import io.burt.jmespath.gson.GsonRuntime;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;


public class MessageAsserter {

    private final JmesPath<JsonElement> path;

    private final JsonElement message;

    public MessageAsserter(String rawMessage) {
        path = new GsonRuntime();
        message = new GsonBuilder().create().fromJson(rawMessage, JsonObject.class);

        verifyBasics();
    }

    private void verifyBasics() {
        verifyTimestampSet();
        verifyFooterSet();
    }
    private void verifyTimestampSet() {
        JsonElement actualText = getJsonElement("Message.timestamp");
        assertTrue(1606503490 < actualText.getAsLong());
    }

    private void verifyFooterSet() {
        JsonElement actualText = getJsonElement("Message.version");
        assertTrue(actualText.getAsString().startsWith("alert v"));
    }

    public MessageAsserter hasText(String text) {
        return verifyJmesPath("Message.text", text);
    }

    public MessageAsserter hasTitle(String title) {
        return verifyJmesPath("Message.title", title);
    }

    public MessageAsserter hasChannel(String channel) {
        return verifyJmesPath("Message.channel", channel);
    }

    public MessageAsserter hasTitleLink(String titleLink) {
        return verifyJmesPath("Message.link", titleLink);
    }

    public MessageAsserter hasColor(String color) {
        return verifyJmesPath("Message.level", color);
    }

    public MessageAsserter hasField(String key, String value) {
        return verifyJmesPath("Message.fields." + key, value);
    }

    private MessageAsserter verifyJmesPath(String jmesPath, String content) {
        JsonElement actualText = getJsonElement(jmesPath);
        assertEquals(content, actualText.getAsString());
        return this;
    }

    private JsonElement getJsonElement(String s) {
        Expression<JsonElement> matcher = path.compile(s);
        return matcher.search(message);
    }
}
