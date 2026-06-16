<details>
<summary>XSD contract: <code>RepresentationType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="RepresentationType" abstract="true">
		<xs:annotation>
			<xs:documentation>RepresentationType is an abstract type that defines a representation. Because the type of item schemes that are allowed as the enumeration vary based on the object in which this is defined, this type is abstract to force that the enumeration reference be restricted to the proper type of item scheme reference.</xs:documentation>
		</xs:annotation>
		<xs:choice>
			<xs:element name="TextFormat" type="TextFormatType">
				<xs:annotation>
					<xs:documentation>TextFormat describes an uncoded textual format.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:sequence>
				<xs:element name="Enumeration" type="common:AnyCodelistReferenceType">
					<xs:annotation>
						<xs:documentation>Enumeration references an item scheme that enumerates the allowable values for this representation.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="EnumerationFormat" type="CodedTextFormatType" minOccurs="0">
					<xs:annotation>
						<xs:documentation>EnumerationFormat describes the facets of the item scheme enumeration. This is for the most part, informational.</xs:documentation>
					</xs:annotation>
				</xs:element>
			</xs:sequence>
		</xs:choice>
		<xs:attribute name="minOccurs" type="xs:nonNegativeInteger" use="optional" default="1">
			<xs:annotation>
				<xs:documentation>The minOccurs attribute indicates the minimum number of values that must be reported for the component.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
		<xs:attribute name="maxOccurs" type="common:OccurenceType" use="optional">
			<xs:annotation>
				<xs:documentation>The maxOccurs attribute indicates the maximum number of values that can be reported for the component.</xs:documentation>
			</xs:annotation>
		</xs:attribute>
	</xs:complexType>
```

</details>
