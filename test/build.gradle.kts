plugins {
    id("java")
}

repositories {
    mavenLocal()
    maven {
        url = uri("https://repo.maven.apache.org/maven2/")
    }
}

dependencies {
    testImplementation("com.google.code.gson:gson:2.8.6")
    testImplementation("org.apache.logging.log4j:log4j-api:2.13.3")
    testImplementation("org.apache.logging.log4j:log4j-core:2.13.3")
    testImplementation("org.apache.logging.log4j:log4j-web:2.13.3")
    testImplementation("org.apache.logging.log4j:log4j-slf4j-impl:2.13.3")
    testImplementation("commons-io:commons-io:2.7")
    testImplementation("com.kohlschutter.junixsocket:junixsocket-core:2.3.2")
    testImplementation("io.burt:jmespath-core:0.5.0")
    testImplementation("io.burt:jmespath-gson:0.5.0")
    testImplementation("org.junit.jupiter:junit-jupiter:5.7.0")
    testImplementation("org.mock-server:mockserver-junit-jupiter:5.11.1")
}

group = "de.njsm.alerter"
version = "1.0"
description = "test"
java.sourceCompatibility = JavaVersion.VERSION_17

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
}

tasks.withType<Test> {
    useJUnitPlatform()
    systemProperty("de.njsm.alerter.config", "$projectDir/src/test/resources/config.yml")
    systemProperty("de.njsm.alerter.client.config", "$projectDir/src/test/resources/client_config.yml")
    systemProperty("de.njsm.alerter.logconfig", "$projectDir/src/test/resources/log4rs.yml")
    systemProperty("de.njsm.alerter.client.binary", "$projectDir/../target/debug/alert")
    systemProperty("de.njsm.alerter.server.binary", "$projectDir/../target/debug/alerter")
}
