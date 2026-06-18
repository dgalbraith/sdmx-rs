<details>
<summary>XSD contract: <code>AttributeListType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AttributeListType">
		<xs:annotation>
			<xs:documentation>AttributeListType describes the attribute descriptor for the data structure definition.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="AttributeListBaseType">
				<xs:choice maxOccurs="unbounded">
					<xs:element ref="Attribute"/>
					<xs:element ref="MetadataAttributeUsage"/>
				</xs:choice>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
