<details>
<summary>XSD contract: <code>CodeType</code> (SDMX 3.1)</summary>

```xml
	<xs:complexType name="CodeType">
		<xs:annotation>
			<xs:documentation>CodeType describes the structure of a code. A code is defined as a language independent set of letters, numbers or symbols that represent a concept whose meaning is described in a natural language. Presentational information not present may be added through the use of annotations.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="ItemType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element ref="common:Name" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>Name provides for a human-readable name for the object. This may be provided in multiple, parallel language-equivalent forms. It may be used in place of a short description.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element ref="common:Description" minOccurs="0" maxOccurs="unbounded">
						<xs:annotation>
							<xs:documentation>Description provides a plain text, human-readable description of the code. This may be provided in multiple, parallel language-equivalent forms.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:choice minOccurs="0">
						<xs:element name="Parent" type="common:IDType">
							<xs:annotation>
								<xs:documentation>Parent provides the ability to describe simple hierarchies within a single codelist, by referencing the id value of another code in the same codelist.</xs:documentation>
							</xs:annotation>
						</xs:element>
					</xs:choice>
				</xs:sequence>
				<xs:attribute name="urn" type="common:CodeUrnType" use="optional"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
