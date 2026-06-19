<details>
<summary>XSD contract: <code>MeasureRelationshipType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="MeasureRelationshipType">
		<xs:annotation>
			<xs:documentation>MeasureRelationshipType allows for the description of an attribute's relationship to one or more measures.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="Measure" type="common:NCNameIDType" maxOccurs="unbounded">
				<xs:annotation>
					<xs:documentation>This is a reference to a measure defined in this data structure definition.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
	</xs:complexType>
```

</details>
