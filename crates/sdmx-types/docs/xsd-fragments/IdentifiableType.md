<details>
<summary>XSD contract: <code>IdentifiableType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="IdentifiableType" abstract="true">
		<xs:annotation>
			<xs:documentation>IdentifiableType is an abstract base type for all identifiable objects.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="AnnotableType">
				<xs:sequence>
					<xs:element ref="Link" minOccurs="0" maxOccurs="unbounded"/>
				</xs:sequence>
				<xs:attribute name="id" type="IDType" use="optional">
					<xs:annotation>
						<xs:documentation>The id is the identifier for the object.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="urn" type="UrnType" use="optional">
					<xs:annotation>
						<xs:documentation>The urn attribute holds a valid SDMX Registry URN (see SDMX Registry Specification for details).</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="uri" type="xs:anyURI" use="optional">
					<xs:annotation>
						<xs:documentation>The uri attribute holds a URI that contains a link to a resource with additional information about the object, such as a web page. This uri is not a SDMX message.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
