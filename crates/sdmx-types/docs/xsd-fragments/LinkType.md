<details>
<summary>XSD contract: <code>LinkType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="LinkType">
		<xs:attribute name="rel" type="xs:string" use="required">
			<xs:annotation>
				<xs:documentation>The type of object that is being linked to.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="url" type="xs:anyURI" use="required">
			<xs:annotation>
				<xs:documentation>The url of the object being linked.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="urn" type="xs:anyURI" use="optional">
			<xs:annotation>
				<xs:documentation>A SDMX registry urn of the object being linked (if applicable).</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="type" type="xs:string" use="optional">
			<xs:annotation>
				<xs:documentation>The type of link (e.g. PDF, text, HTML, reference metadata).</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
